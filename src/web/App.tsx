import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { ApiAnswer, ApiGame, ApiGameStats, convertApiBoardToBoard, gameApi } from './api/gameApi';
import AnswerSection from './components/AnswerSection';
import Board from './components/Board';
import HeatmapModal from './components/HeatmapModal';
// Lazy load word list to avoid blocking initial render
import PathfinderLogo from './components/Logo';
import { useUser } from './hooks/useUser';
import { useVirtualKeyboard } from './hooks/useVirtualKeyboard';
import { generateBoard } from './utils/boardGeneration';
import { convertConstraintSetsToConstraints, mergeAllAnswerGroupConstraintSets, UnsatisfiableConstraint } from './utils/constraintResolution';
import { AnswerGroupConstraintSet, Position, Tile } from './utils/models';
import { Answer, findAllPaths, findBestPath, findPathsForHighlighting } from './utils/pathfinding';
import { scoreAnswerGroup } from './utils/scoring';


function App() {
  const { sequenceNumber } = useParams<{ sequenceNumber: string }>();
  const navigate = useNavigate();
  const { user, isLoading: userLoading, clearUser } = useUser();
  const { isVisible: isVirtualKeyboardVisible } = useVirtualKeyboard();
  const [board, setBoard] = useState<Tile[][]>([]);
  const [answers, setAnswers] = useState<string[]>(['', '', '', '', '']);
  const [validAnswers, setValidAnswers] = useState<boolean[]>([false, false, false, false, false]);
  const [scores, setScores] = useState<number[]>([0, 0, 0, 0, 0]);
  const [wildcardConstraints, setWildcardConstraints] = useState<Record<string, string>>({});
  const [highlightedPaths, setHighlightedPaths] = useState<Position[][]>([]);
  const [currentInputIndex, setCurrentInputIndex] = useState<number>(-1);
  const [showHeatmapModal, setShowHeatmapModal] = useState<boolean>(false);
  const [gameStats, setGameStats] = useState<ApiGameStats | null>(null);
  const [validPaths, setValidPaths] = useState<(Position[] | null)[]>([]);
  const [currentGame, setCurrentGame] = useState<ApiGame | null>(null);
  const [isLoadingGame, setIsLoadingGame] = useState(true);
  const [apiError, setApiError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isValidWordLoaded, setIsValidWordLoaded] = useState(false);
  const [isValidWordFn, setIsValidWordFn] = useState<((word: string) => boolean) | null>(null);
  const [isGameCompleted, setIsGameCompleted] = useState(false);

  // Load word validation function asynchronously
  useEffect(() => {
    const loadWordList = async () => {
      // Try to load game-specific word list first if we have a game
      if (currentGame) {
        try {
          const gameWords = await gameApi.getGameWords(currentGame.id);
          if (gameWords.length > 0) {
            const gameWordSet = new Set(gameWords.map(word => word.toLowerCase()));
            setIsValidWordFn(() => (word: string) => gameWordSet.has(word.toLowerCase()));
            setIsValidWordLoaded(true);
            return;
          }
        } catch (error) {
          console.warn('Failed to load game-specific word list, falling back to complete word list:', error);
        }
      }

      // Fallback to complete word list
      try {
        const { isValidWord } = await import('./data/wordList');
        setIsValidWordFn(() => isValidWord);
        setIsValidWordLoaded(true);
      } catch (error) {
        console.error('Failed to load word list:', error);
        // Fallback: allow all words if word list fails to load
        setIsValidWordFn(() => () => true);
        setIsValidWordLoaded(true);
      }
    };
    loadWordList();
  }, [currentGame]);

  // Load game immediately, don't wait for user
  useEffect(() => {
    // Clear highlighting state when switching puzzles
    setHighlightedPaths([]);
    setCurrentInputIndex(-1);
    setIsGameCompleted(false);
    // Reset word list state when switching games
    setIsValidWordLoaded(false);
    setIsValidWordFn(null);
    loadGame();
  }, [sequenceNumber]);

  // Load existing game entry when user becomes available
  useEffect(() => {
    if (!userLoading && user && currentGame) {
      loadExistingGameEntry();
    }
  }, [userLoading, user, currentGame]);

  const loadGame = async () => {
    try {
      setIsLoadingGame(true);
      setApiError(null);
      const game = sequenceNumber 
        ? await gameApi.getGameBySequence(parseInt(sequenceNumber))
        : await gameApi.getDailyGame();
      console.log('Received game:', game);
      console.log('Sequence number:', game.sequence_number);
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

  const loadExistingGameEntry = async () => {
    if (!user || !currentGame) return;
    
    try {
      const gameEntry = await gameApi.getGameEntry(
        currentGame.id, 
        user.user_id, 
        user.cookie_token
      );
      
      if (gameEntry && gameEntry.answers.length > 0) {
        // Set completion status
        setIsGameCompleted(gameEntry.completed);
        
        // Set game stats if available (for completed games)
        if (gameEntry.stats) {
          setGameStats(gameEntry.stats);
        }
        
        // Populate answers from existing game entry
        const loadedAnswers = ['', '', '', '', ''];
        
        // Load existing answers
        gameEntry.answers.forEach((answer, index) => {
          if (index < 5) {
            loadedAnswers[index] = answer.word;
          }
        });
        
        setAnswers(loadedAnswers);
        
        // Re-validate all answers using the new permissive logic
        const validation = validateAllAnswers(loadedAnswers);
        
        setValidAnswers(validation.validAnswers);
        setScores(validation.scores);
        setValidPaths(validation.paths);
        setWildcardConstraints(convertConstraintSetsToConstraints(validation.constraintSets, board));
      } else {
        setIsGameCompleted(false);
        setGameStats(null);
        setValidAnswers([false, false, false, false, false]);
        setScores([0,0,0,0,0]);
        setValidPaths([]);
        setAnswers(['','','','','']);
        setWildcardConstraints({});
      }
    } catch (error) {
      console.warn('Failed to load existing game entry:', error);
      // If this is a 401 error, the user is invalid, clear localStorage
      if (error instanceof Error && error.message.includes('401')) {
        console.log('User appears to be invalid, clearing localStorage and creating new user');
        clearUser();
      }
    }
  };

  const saveProgress = async () => {
    if (!user || !currentGame || isGameCompleted) return;
    
    try {
      // Convert all valid answers to API format with scores
      const apiAnswers: ApiAnswer[] = answers
        .map((word, index) => {
          if (!word || !validAnswers[index]) return null;
          
          return {
            word,
            score: scores[index]
          };
        })
        .filter((answer): answer is ApiAnswer => answer !== null);

      if (apiAnswers.length > 0) {
        await gameApi.updateProgress({
          user_id: user.user_id,
          cookie_token: user.cookie_token,
          answers: apiAnswers,
          game_id: currentGame.id,
        });
      }
    } catch (error) {
      console.warn('Failed to save progress:', error);
    }
  };

  const handleAnswerBlur = (_index: number): void => {
    // Save progress when any answer input loses focus
    if (user && currentGame && !isGameCompleted) {
      saveProgress();
    }
  };

  // Validate all answers together - skips invalid words and continues with remaining valid words
  const validateAllAnswers = (allAnswers: string[], focusedIndex: number = -1): { validAnswers: boolean[], scores: number[], paths: (Position[] | null)[], constraintSets: AnswerGroupConstraintSet } => {
    const validAnswers = [false, false, false, false, false];
    const scores = [0, 0, 0, 0, 0];
    const paths: (Position[] | null)[] = [null, null, null, null, null];
    const usedWords = new Set<string>();
    
    // Step 1: Sanitize inputs (convert to lowercase)
    const sanitizedAnswers = allAnswers.map(word => word.toLowerCase());
    
    // Step 2: Collect valid words that pass dictionary and path validation
    interface ValidWordInfo {
      index: number;
      originalWord: string;
      sanitizedWord: string;
      answer: Answer;
      isInvalid?: boolean; // Track if this word is invalid (focused input with validation failures)
    }
    
    const validWordsInfo: ValidWordInfo[] = [];

    function passesBasicValidityChecks(sanitizedWord: string): boolean {
      // Skip empty or too short words
      if (!sanitizedWord || sanitizedWord.length < 2) {
        return false;
      }
      
      // Skip if word list is loaded and word is not in dictionary
      if (isValidWordLoaded && isValidWordFn && !isValidWordFn(sanitizedWord)) {
        return false; // Skip this word, don't fail entire validation
      }
      
      return true;
    }
    
    for (let i = 0; i < sanitizedAnswers.length; i++) {
      const originalWord = allAnswers[i];
      const sanitizedWord = sanitizedAnswers[i];
      const isFocused = i === focusedIndex;
      
      // skip certain validation if it is for the current input, to show proper pathfinding
      if (!isFocused && !passesBasicValidityChecks(sanitizedWord)) {
        continue;
      }

      // Check for duplicate words
      if (usedWords.has(sanitizedWord)) {
        continue; // Skip duplicate
      }
      
      // Get all paths for this word using findAllPaths
      const answer = findAllPaths(board, sanitizedWord);
      
      if (answer.paths.length === 0) {
        continue; // Skip this word, don't fail entire validation
      }
      
      // This word is valid - add to our collection
      validWordsInfo.push({
        index: i,
        originalWord,
        sanitizedWord,
        answer
      });
      usedWords.add(sanitizedWord);
    }
    
    // Step 3: Check if the valid words can coexist (constraint compatibility)
    if (validWordsInfo.length === 0) {
      // No valid words at all
      return {
        validAnswers,
        scores,
        paths,
        constraintSets: { pathConstraintSets: [] }
      };
    }
    
    const constraintSets = validWordsInfo.map(info => info.answer.constraintsSet);
    
    let isValidSet = true;
    let finalConstraintSet: AnswerGroupConstraintSet = { pathConstraintSets: [] };
    
    try {
      finalConstraintSet = mergeAllAnswerGroupConstraintSets(constraintSets);
    } catch (e) {
      if (e instanceof UnsatisfiableConstraint) {
        isValidSet = false;
      } else {
        throw e;
      }
    }
    
    if (!isValidSet) {
      // Valid words exist but their constraints conflict - return no valid answers
      return {
        validAnswers,
        scores,
        paths,
        constraintSets: { pathConstraintSets: [] }
      };
    }
    
    // Step 4: All valid words can coexist - populate results
    // Use scoreAnswerGroup to get optimal scores for all words
    const validWords = validWordsInfo.map(info => info.sanitizedWord);
    const { scores: scoresByWord, optimalConstraintSets } = scoreAnswerGroup(validWords, board);
    
    // Use the optimal constraint sets from scoreAnswerGroup instead of manually merging
    finalConstraintSet = { pathConstraintSets: optimalConstraintSets };
    
    for (const info of validWordsInfo) {
      const { index, originalWord, sanitizedWord, isInvalid } = info;
      
      // Find the best path for this word
      const bestPath = findBestPath(board, originalWord, {});
      if (!bestPath) continue; // Should not happen since we already found paths
      
      validAnswers[index] = true;
      
      // For focused input that fails validation, set score to 0
      scores[index] = isInvalid ? 0 : scoresByWord[sanitizedWord] || 0;
      
      paths[index] = bestPath;
    }

    if (!passesBasicValidityChecks(sanitizedAnswers[focusedIndex])) {
      validAnswers[focusedIndex] = false;
      scores[focusedIndex] = 0;
    }

    return {
      validAnswers,
      scores,
      paths,
      constraintSets: finalConstraintSet
    };
  };


  const handleAnswerInputChange = (index: number, value?: string): void => {
    let shouldHighlight = true;

    // If value is provided, update the answer (onChange behavior)
    if (value !== undefined) {
      const newAnswers = [...answers];
      newAnswers[index] = value;
      setAnswers(newAnswers);

      // Use new validation that skips invalid words
      const validation = validateAllAnswers(newAnswers, index);
      shouldHighlight = validation.constraintSets.pathConstraintSets.length > 0
      if (shouldHighlight) {
        const constraints = convertConstraintSetsToConstraints(validation.constraintSets, board);
        
        setValidAnswers(validation.validAnswers);
        setScores(validation.scores);
        setWildcardConstraints(constraints);
        setValidPaths(validation.paths);
      }
    }

    if (!shouldHighlight) {
      setHighlightedPaths([]);
      return;
    }

    // Update highlighting (both onChange and onFocus behavior)
    const currentValue = value !== undefined ? value : answers[index];
    
    // Update highlighting when focusing on a different input or when value changes
    if (currentInputIndex !== index || value !== undefined) {
      setCurrentInputIndex(index);
      
      // Set highlighted paths for the current input
      if (currentValue && currentValue.length > 0) {
        const currentConstraints = value !== undefined 
          ? convertConstraintSetsToConstraints(validateAllAnswers([...answers.slice(0, index), value, ...answers.slice(index + 1)], index).constraintSets, board)
          : wildcardConstraints;
        const paths = findPathsForHighlighting(board, currentValue, currentConstraints);
        setHighlightedPaths(paths);
      } else {
        setHighlightedPaths([]);
      }
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
    // If game is already completed or offline, just show stats modal
    if (isGameCompleted || !currentGame || isSubmitting) {
      setShowHeatmapModal(true);
      return;
    }

    setIsSubmitting(true);
    try {
      // Convert frontend answers to API format
      const apiAnswers: ApiAnswer[] = answers
        .map((word, index) => {
          if (!validAnswers[index]) return null;
          
          return {
            word,
            score: scores[index]
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
        answers: apiAnswers,
        game_id: currentGame.id,
      });

      // Mark game as completed after successful submission
      setIsGameCompleted(true);
      setGameStats(response.stats);
      setShowHeatmapModal(true);
    } catch (error) {
      console.error('Failed to submit answers:', error);
      setApiError('Failed to submit answers. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handlePreviousPuzzle = (): void => {
    if (!currentGame || currentGame.sequence_number <= 1) return;
    const prevSequence = currentGame.sequence_number - 1;
    navigate(`/puzzle/${prevSequence}`);
  };

  const handleNextPuzzle = (): void => {
    if (!currentGame) return;
    const nextSequence = currentGame.sequence_number + 1;
    navigate(`/puzzle/${nextSequence}`);
  };

  const isNextDisabled = (): boolean => {
    if (!currentGame) return true;
    
    // Get today's date in UTC (midnight)
    const today = new Date();
    const todayUTC = new Date(Date.UTC(today.getFullYear(), today.getMonth(), today.getDate()));
    
    // Parse game date as UTC (assumes currentGame.date is YYYY-MM-DD format)
    const currentGameDate = new Date(currentGame.date + 'T00:00:00.000Z');
    
    // Compare UTC dates
    return currentGameDate >= todayUTC;
  };

  const LoadingSpinner = () => (
    <>
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
    </>
  );

  // Only show loading if both user and game are loading
  if (userLoading && isLoadingGame) {
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
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div style={{ 
      fontFamily: 'Arial, sans-serif', 
      maxWidth: '300px', 
      margin: '0 auto', 
      padding: '20px',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center'
    }}>
      <div style={{
        display: isVirtualKeyboardVisible ? 'none' : 'block'
      }}>
        <PathfinderLogo />
        
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

        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          marginTop: '20px',
          marginBottom: '20px',
          gap: '16px'
        }}>
        <button
          onClick={handlePreviousPuzzle}
          disabled={!currentGame || currentGame.sequence_number <= 1}
          style={{
            backgroundColor: (!currentGame || currentGame.sequence_number <= 1) ? '#cccccc' : '#4CAF50',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            padding: '8px 12px',
            fontSize: '14px',
            cursor: (!currentGame || currentGame.sequence_number <= 1) ? 'not-allowed' : 'pointer',
            transform: 'scale(1)',
            transition: 'all 0.1s ease',
            boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
          }}
          onMouseDown={(e) => {
            if (!(!currentGame || currentGame.sequence_number <= 1)) {
              e.currentTarget.style.transform = 'scale(0.95)';
            }
          }}
          onMouseUp={(e) => {
            if (!(!currentGame || currentGame.sequence_number <= 1)) {
              e.currentTarget.style.transform = 'scale(1)';
            }
          }}
          onMouseLeave={(e) => {
            if (!(!currentGame || currentGame.sequence_number <= 1)) {
              e.currentTarget.style.transform = 'scale(1)';
            }
          }}
        >
          ←
        </button>
        
        <div style={{
          textAlign: 'center',
          color: '#666',
          fontSize: '16px',
          minWidth: '200px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: '8px'
        }}>
          {isLoadingGame ? (
            <>
              <span>Puzzle #</span>
              {!sequenceNumber && (
                <span> · {new Date().toISOString().split('T')[0]}</span>
              )}
            </>
          ) : (
            <>Puzzle #{currentGame?.sequence_number || 'N/A'} · {currentGame?.date}</>
          )}
        </div>
        
        {!isNextDisabled() && currentGame ? (
          <button
            onClick={handleNextPuzzle}
            style={{
              backgroundColor: '#4CAF50',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              padding: '8px 12px',
              fontSize: '14px',
              cursor: 'pointer',
              transform: 'scale(1)',
              transition: 'all 0.1s ease',
              boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
            }}
            onMouseDown={(e) => {
              e.currentTarget.style.transform = 'scale(0.95)';
            }}
            onMouseUp={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
            }}
          >
            →
          </button>
        ) : (
          <div style={{
            width: '40px',
            height: '32px'
          }} />
        )}
      </div>
      </div>
      
      {board.length > 0 && (
        <Board 
          board={board} 
          highlightedPaths={highlightedPaths}
          wildcardConstraints={wildcardConstraints}
          answers={answers}
          validAnswers={validAnswers}
          currentWord={currentInputIndex >= 0 ? answers[currentInputIndex] : ''}
        />
      )}
      
      {board.length === 0 && isLoadingGame && (
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '300px',
          gap: '16px'
        }}>
          <span>Loading board...</span>
          <LoadingSpinner />
        </div>
      )}
      
      <AnswerSection
        answers={answers}
        onAnswerChange={(index, value) => handleAnswerInputChange(index, value)}
        validAnswers={validAnswers}
        scores={scores}
        onSubmit={handleSubmit}
        onAnswerFocus={(index) => handleAnswerInputChange(index)}
        onAnswerBlur={handleAnswerBlur}
        isSubmitting={isSubmitting}
        isWordListLoading={!isValidWordLoaded}
        isGameCompleted={isGameCompleted}
        isOffline={!!apiError}
        board={board}
        validPaths={validPaths}
      />
      
      <HeatmapModal
        isOpen={showHeatmapModal}
        onClose={() => setShowHeatmapModal(false)}
        tileUsage={calculateTileUsage()}
        board={board}
        totalScore={scores.reduce((sum, score) => sum + score, 0)}
        scores={scores}
        gameStats={gameStats}
      />
    </div>
  );
}

export default App;
