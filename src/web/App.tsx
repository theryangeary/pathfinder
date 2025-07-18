import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { ApiAnswer, ApiGame, ApiGameStats, convertApiBoardToBoard, gameApi } from './api/gameApi';
import AnswerSection from './components/AnswerSection';
import Board from './components/Board';
import HeatmapModal from './components/HeatmapModal';
// Lazy load word list to avoid blocking initial render
import PathfinderLogo from './components/Logo';
import SEOHead from './components/SEOHead';
import Tutorial from './components/Tutorial';
import { useMobileDetection } from './hooks/useMobileDetection';
import { useUser } from './hooks/useUser';
import { useVirtualKeyboard } from './hooks/useVirtualKeyboard';
import { generateBoard } from './utils/boardGeneration';
import { Tile } from './utils/models';
import { findPathsForHighlighting } from './utils/pathfinding';
import { validateAllAnswers } from './utils/validation';


function App() {
  const { sequenceNumber } = useParams<{ sequenceNumber: string }>();
  const navigate = useNavigate();
  const { user, isLoading: userLoading, showTutorial, clearUser, completeTutorial } = useUser();
  const { isVisible: isVirtualKeyboardVisible } = useVirtualKeyboard();
  const isMobile = useMobileDetection();

  const [board, setBoard] = useState<Tile[][]>([]);
  const [answers, setAnswers] = useState<string[]>(['', '', '', '', '']);
  const [currentInputIndex, setCurrentInputIndex] = useState<number>(-1);
  const [showHeatmapModal, setShowHeatmapModal] = useState<boolean>(false);
  const [gameStats, setGameStats] = useState<ApiGameStats | null>(null);
  const [currentGame, setCurrentGame] = useState<ApiGame | null>(null);
  const [isLoadingGame, setIsLoadingGame] = useState(true);
  const [apiError, setApiError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isValidWordLoaded, setIsValidWordLoaded] = useState(false);
  const [isValidWordFn, setIsValidWordFn] = useState<((word: string) => boolean) | null>(null);
  const [isGameCompleted, setIsGameCompleted] = useState(false);

  const shouldUseCompactLayout = isMobile && currentInputIndex !== -1 && isVirtualKeyboardVisible;

  // Derived state calculations
  const validation = useMemo(() => {
    if (board.length === 0 || !isValidWordLoaded) {
      return {
        validAnswers: [false, false, false, false, false],
        scores: [0, 0, 0, 0, 0],
        paths: [null, null, null, null, null],
        constraintSets: { pathConstraintSets: [] }
      };
    }
    const result = validateAllAnswers(board, answers, isValidWordLoaded, isValidWordFn, currentInputIndex);
    if (result.constraintSets.pathConstraintSets.length === 0) {
      // the currently editted word can't fit with the rest; get validation but ignore this word
      const answersExceptCurrent = [...answers.slice(0, currentInputIndex), "", ...answers.slice(currentInputIndex+1)]
      return validateAllAnswers(board, answersExceptCurrent, isValidWordLoaded, isValidWordFn);
    }
    return result
  }, [board, answers, isValidWordLoaded, isValidWordFn]);

  const { validAnswers, scores, paths: validPaths, constraintSets: wildcardConstraints } = validation;

  const highlightedPaths = useMemo(() => {
    if (currentInputIndex === -1 || !answers[currentInputIndex] || answers[currentInputIndex].length === 0) {
      return [];
    }
    if (wildcardConstraints.pathConstraintSets.length === 0) {
      return [];
    }
    return findPathsForHighlighting(board, answers[currentInputIndex], wildcardConstraints);
  }, [board, answers, currentInputIndex, wildcardConstraints]);

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

  // Save progress each time input changes
  useEffect(() => {
    saveProgress()
  }, [currentInputIndex])

  const loadGame = async () => {
    try {
      setIsLoadingGame(true);
      setApiError(null);
      const game = sequenceNumber 
        ? await gameApi.getGameBySequence(parseInt(sequenceNumber))
        : await gameApi.getDailyGame();
      setCurrentGame(game);
      const newBoard = convertApiBoardToBoard(game.board);
      setBoard(newBoard);
    } catch (error) {
      console.error('Failed to load daily game from API, falling back to local generation:', error);
      setApiError('reload');
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
      } else {
        setIsGameCompleted(false);
        setGameStats(null);
        setAnswers(['','','','','']);
      }
    } catch (error) {
      console.warn('Failed to load existing game entry:', error);
      // If this is a 401 error, the user is invalid, clear localStorage
      if (error instanceof Error && error.message.includes('401')) {
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
          completed: false
        });
      }
    } catch (error) {
      console.warn('Failed to save progress:', error);
    }
  };

  const handleAnswerBlur = (_index: number): void => {
    setCurrentInputIndex(-1);
  };

  const handleAnswerInputChange = (index: number, value?: string): void => {
    // If value is provided, update the answer (onChange behavior)
    if (value !== undefined) {
      const newAnswers = [...answers];
      newAnswers[index] = value;
      setAnswers(newAnswers);
    }

    // Update highlighting when focusing on a different input or when value changes
    if (currentInputIndex !== index || value !== undefined) {
      setCurrentInputIndex(index);
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
        completed: true,
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
    <>
      <SEOHead
        gameDate={currentGame?.date}
        isCompleted={isGameCompleted}
        totalScore={scores.reduce((sum, score) => sum + score, 0)}
      />
      <div style={{ 
        fontFamily: 'Arial, sans-serif', 
        maxWidth: '400px', 
        height: '100vh',
        margin: '0 auto', 
        padding: shouldUseCompactLayout ? '5px' : '20px',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center'
      }}>
      { !shouldUseCompactLayout && 
      <div style={{
        display: 'block'
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
            {apiError === 'reload' ? (
              <>Failed to connect to server. Please <a href="" onClick={() => window.location.reload()}>reload</a></>
            ) : (
              apiError
            )}
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
          ) : currentGame ? (
            <>Puzzle #{currentGame.sequence_number} · {currentGame.date}</>
          ) : (
            <>Puzzle #N/A</>
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
    }
      
      {board.length > 0 && (
        <Board 
          board={board} 
          highlightedPaths={highlightedPaths}
          wildcardConstraints={wildcardConstraints}
          shouldUseCompactLayout={shouldUseCompactLayout}
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
        onViewAllAnswers={() => setCurrentInputIndex(-1)}
        isSubmitting={isSubmitting}
        isWordListLoading={!isValidWordLoaded}
        isGameCompleted={isGameCompleted}
        shouldUseCompactLayout={shouldUseCompactLayout}
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

      <Tutorial
        isOpen={showTutorial}
        onClose={completeTutorial}
        onSkip={completeTutorial}
      />
      </div>
    </>
  );
}

export default App;
