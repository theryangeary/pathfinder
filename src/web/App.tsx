import { useEffect, useState } from 'react';
import { ApiAnswer, ApiGame, convertApiBoardToBoard, gameApi } from './api/gameApi';
import AnswerSection from './components/AnswerSection';
import Board from './components/Board';
import HeatmapModal from './components/HeatmapModal';
import { isValidWord } from './data/wordList';
import { useUser } from './hooks/useUser';
import { generateBoard } from './utils/boardGeneration';
import { findBestPath, findPathsForHighlighting, getWildcardConstraintsFromPath } from './utils/pathfinding';
import { calculateWordScore, Position, Tile } from './utils/scoring';

interface ValidationResult {
  isValid: boolean;
  score: number;
  path: Position[] | null;
  newConstraints?: Record<string, string>;
}

function App() {
  const { user, isLoading: userLoading } = useUser();
  const [board, setBoard] = useState<Tile[][]>([]);
  const [answers, setAnswers] = useState<string[]>(['', '', '', '', '']);
  const [validAnswers, setValidAnswers] = useState<boolean[]>([false, false, false, false, false]);
  const [scores, setScores] = useState<number[]>([0, 0, 0, 0, 0]);
  const [wildcardConstraints, setWildcardConstraints] = useState<Record<string, string>>({});
  const [highlightedPaths, setHighlightedPaths] = useState<Position[][]>([]);
  const [currentInputIndex, setCurrentInputIndex] = useState<number>(-1);
  const [showHeatmapModal, setShowHeatmapModal] = useState<boolean>(false);
  const [validPaths, setValidPaths] = useState<(Position[] | null)[]>([]);
  const [currentGame, setCurrentGame] = useState<ApiGame | null>(null);
  const [isLoadingGame, setIsLoadingGame] = useState(true);
  const [apiError, setApiError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!userLoading) {
      loadDailyGame();
    }
  }, [userLoading]);

  const loadDailyGame = async () => {
    try {
      setIsLoadingGame(true);
      setApiError(null);
      const game = await gameApi.getDailyGame();
      setCurrentGame(game);
      const newBoard = convertApiBoardToBoard(game.board);
      setBoard(newBoard);
    } catch (error) {
      console.error('Failed to load daily game from API, falling back to local generation:', error);
      setApiError('Failed to connect to server. Playing offline.');
      // Fallback to local board generation
      const newBoard = generateBoard();
      setBoard(newBoard);
    } finally {
      setIsLoadingGame(false);
    }
  };

  const validateAnswer = (word: string, _answerIndex: number, currentConstraints: Record<string, string>, previousAnswers: string[] = []): ValidationResult => {
    if (!word || word.length < 2) return { isValid: false, score: 0, path: null };
    
    if (!isValidWord(word)) return { isValid: false, score: 0, path: null };
    
    // Check if this word was already used in a previous answer slot
    if (previousAnswers.includes(word.toLowerCase())) {
      return { isValid: false, score: 0, path: null };
    }
    
    const path = findBestPath(board, word, currentConstraints);
    if (!path) return { isValid: false, score: 0, path: null };
    
    const newConstraints = getWildcardConstraintsFromPath(board, word, path);
    
    for (const [key, value] of Object.entries(newConstraints)) {
      if (currentConstraints[key] && currentConstraints[key] !== value) {
        return { isValid: false, score: 0, path: null };
      }
    }
    
    const score = calculateWordScore(word, path, board);
    return { isValid: true, score, path, newConstraints };
  };

  const handleAnswerFocus = (index: number): void => {
    // Clear highlighting when focusing on a different input
    if (currentInputIndex !== index) {
      setHighlightedPaths([]);
      setCurrentInputIndex(index);
    }
  };

  const handleAnswerChange = (index: number, value: string): void => {
    const newAnswers = [...answers];
    newAnswers[index] = value;
    setAnswers(newAnswers);

    // Reset constraints and rebuild from scratch based on all valid answers
    let tempConstraints = {};
    const newValidAnswers = [...validAnswers];
    const newScores = [...scores];
    const newValidPaths = [...validPaths];

    // Validate all answers from the beginning, rebuilding constraints as we go
    const validPreviousAnswers = [];
    for (let i = 0; i < 5; i++) {
      if (newAnswers[i]) {
        const result = validateAnswer(newAnswers[i], i, tempConstraints, validPreviousAnswers);
        newValidAnswers[i] = result.isValid;
        newScores[i] = result.score;
        
        if (result.isValid && result.newConstraints) {
          tempConstraints = { ...tempConstraints, ...result.newConstraints };
          // Add this valid answer to the list of previous answers for future validation
          validPreviousAnswers.push(newAnswers[i].toLowerCase());
          // Store the path for heatmap calculation
          newValidPaths[i] = result.path;
        } else {
          newValidPaths[i] = null;
        }

        // Set highlighted paths only for the current input
        if (i === index) {
          const paths = findPathsForHighlighting(board, newAnswers[i], tempConstraints);
          setHighlightedPaths(paths);
          setCurrentInputIndex(index);
        }
      } else {
        // Clear validation state for empty answers
        newValidAnswers[i] = false;
        newScores[i] = 0;
        newValidPaths[i] = null;
        
        // Clear highlighted paths if this is the current input
        if (i === index) {
          setHighlightedPaths([]);
          setCurrentInputIndex(index);
        }
      }
    }

    setValidAnswers(newValidAnswers);
    setScores(newScores);
    setWildcardConstraints(tempConstraints);
    setValidPaths(newValidPaths);

    if (!value) {
      setHighlightedPaths([]);
      setCurrentInputIndex(-1);
    }
  };

  const calculateTileUsage = (): number[][] => {
    // Initialize 4x4 grid with zeros
    const usage = Array(4).fill(null).map(() => Array(4).fill(0));
    
    // Count usage from all valid paths
    validPaths.forEach(path => {
      if (path) {
        path.forEach(position => {
          usage[position.row][position.col]++;
        });
      }
    });
    
    return usage;
  };

  const handleSubmit = async (): Promise<void> => {
    if (!currentGame || isSubmitting) return;

    setIsSubmitting(true);
    try {
      // Convert frontend answers to API format
      const apiAnswers: ApiAnswer[] = answers
        .map((word, index) => {
          if (!validAnswers[index] || !validPaths[index]) return null;
          
          return {
            word,
            score: scores[index],
            path: validPaths[index]!.map(pos => ({ row: pos.row, col: pos.col })),
            wildcard_constraints: wildcardConstraints
          };
        })
        .filter((answer): answer is ApiAnswer => answer !== null);

      if (apiAnswers.length === 0) {
        console.error('No valid answers to submit');
        setApiError('No valid answers to submit');
        return;
      }

      // Submit to backend
      const response = await gameApi.submitAnswers({
        user_id: user?.user_id,
        cookie_token: user?.cookie_token,
        answers: apiAnswers
      });

      setShowHeatmapModal(true);
    } catch (error) {
      console.error('Failed to submit answers:', error);
      setApiError('Failed to submit answers. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  if (userLoading || isLoadingGame) {
    return (
      <div style={{ 
        fontFamily: 'Arial, sans-serif', 
        maxWidth: '800px', 
        margin: '0 auto', 
        padding: '20px',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '50vh'
      }}>
        <h2>Loading daily puzzle...</h2>
        <div style={{ 
          width: '40px', 
          height: '40px', 
          border: '4px solid #f3f3f3',
          borderTop: '4px solid #3498db',
          borderRadius: '50%',
          animation: 'spin 1s linear infinite'
        }} />
        <style dangerouslySetInnerHTML={{
          __html: `
            @keyframes spin {
              0% { transform: rotate(0deg); }
              100% { transform: rotate(360deg); }
            }
          `
        }} />
      </div>
    );
  }

  return (
    <div style={{ 
      fontFamily: 'Arial, sans-serif', 
      maxWidth: '800px', 
      margin: '0 auto', 
      padding: '20px',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center'
    }}>
      <h1 style={{ textAlign: 'center' }}>Word Game</h1>
      
      {apiError && (
        <div style={{
          backgroundColor: '#fff3cd',
          border: '1px solid #ffeaa7',
          color: '#856404',
          padding: '10px',
          borderRadius: '4px',
          marginBottom: '20px',
          textAlign: 'center'
        }}>
          {apiError}
        </div>
      )}

      {currentGame && (
        <div style={{
          textAlign: 'center',
          marginBottom: '20px',
          color: '#666'
        }}>
          <p>Daily Puzzle: {currentGame.date}</p>
          <p>Game ID: {currentGame.id.slice(0, 8)}...</p>
        </div>
      )}
      
      <Board 
        board={board} 
        highlightedPaths={highlightedPaths}
        wildcardConstraints={wildcardConstraints}
        answers={answers}
        validAnswers={validAnswers}
        currentWord={currentInputIndex >= 0 ? answers[currentInputIndex] : ''}
      />
      
      <AnswerSection
        answers={answers}
        onAnswerChange={handleAnswerChange}
        validAnswers={validAnswers}
        scores={scores}
        onSubmit={handleSubmit}
        onAnswerFocus={handleAnswerFocus}
        isSubmitting={isSubmitting}
      />
      
      <HeatmapModal
        isOpen={showHeatmapModal}
        onClose={() => setShowHeatmapModal(false)}
        tileUsage={calculateTileUsage()}
        board={board}
        totalScore={scores.reduce((sum, score) => sum + score, 0)}
        scores={scores}
      />
    </div>
  );
}

export default App;
