import { useEffect, useRef, useState } from 'react';
import { Position, Tile } from '../utils/models';
import AnswerInput, { AnswerInputHandle } from './AnswerInput';

interface AnswerSectionProps {
  answers: string[];
  onAnswerChange: (index: number, value: string) => void;
  validAnswers: boolean[];
  scores: number[];
  onSubmit: () => void;
  onAnswerFocus: (index: number) => void;
  onAnswerBlur: (index: number) => void;
  isSubmitting?: boolean;
  isWordListLoading?: boolean;
  isGameCompleted?: boolean;
  isOffline?: boolean;
  board: Tile[][];
  validPaths: (Position[] | null)[];
}

function AnswerSection({ 
  answers, 
  onAnswerChange, 
  validAnswers, 
  scores,
  onSubmit,
  onAnswerFocus,
  onAnswerBlur,
  isSubmitting = false,
  isWordListLoading = false,
  isGameCompleted = false,
  isOffline = false,
  board,
  validPaths
}: AnswerSectionProps) {
  const inputRefs = useRef<(AnswerInputHandle | null)[]>([]);
  const [isKeyboardVisible, setIsKeyboardVisible] = useState(false);
  const [currentCarouselIndex, setCurrentCarouselIndex] = useState(0);
  const [isNavigating, setIsNavigating] = useState(false);

  useEffect(() => {
    const detectKeyboard = () => {
      if (window.visualViewport) {
        const initialHeight = window.visualViewport.height;
        let timeoutId: number;
        
        const handleViewportChange = () => {
          // Clear any pending timeout to debounce the detection
          clearTimeout(timeoutId);
          
          timeoutId = setTimeout(() => {
            const currentHeight = window.visualViewport.height;
            const heightDifference = initialHeight - currentHeight;
            const keyboardVisible = heightDifference > 100; // Threshold for keyboard detection
            
            setIsKeyboardVisible(prev => {
              // When switching to carousel mode, find the focused input and set carousel index
              if (keyboardVisible && !prev) {
                const focusedInput = document.activeElement;
                if (focusedInput && focusedInput.tagName === 'INPUT') {
                  // Find which answer input is focused
                  for (let i = 0; i < inputRefs.current.length; i++) {
                    const inputEl = inputRefs.current[i]?.getElement();
                    if (inputEl && document.activeElement === inputEl) {
                      setCurrentCarouselIndex(i);
                      break;
                    }
                  }
                }
              }
              return keyboardVisible;
            });
          }, 150); // Debounce to prevent rapid state changes
        };
        
        window.visualViewport.addEventListener('resize', handleViewportChange);
        return () => {
          clearTimeout(timeoutId);
          window.visualViewport.removeEventListener('resize', handleViewportChange);
        };
      }
    };

    const cleanup = detectKeyboard();
    return cleanup;
  }, []);

  // Maintain focus when switching to/from carousel mode or changing carousel index
  useEffect(() => {
    if (isKeyboardVisible && inputRefs.current[currentCarouselIndex]) {
      // Use multiple attempts to ensure focus is maintained
      const focusInput = () => {
        inputRefs.current[currentCarouselIndex]?.focus();
      };
      
      // Immediate focus
      focusInput();
      
      // Backup focus attempts to ensure keyboard stays open
      setTimeout(focusInput, 0);
      setTimeout(focusInput, 50);
      setTimeout(() => {
        focusInput();
        // Clear navigation flag after focus is complete
        setIsNavigating(false);
      }, 100);
    }
  }, [isKeyboardVisible, currentCarouselIndex]);

  const isInputEnabled = (index: number): boolean => {
    if (index < 0 || index >= answers.length) return false;
    const isValid = validAnswers[index];
    return index === 0 || validAnswers.slice(0, index).every(valid => valid) || isValid;
  };

  const goToPrevious = () => {
    const targetIndex = currentCarouselIndex - 1;
    if (targetIndex >= 0 && isInputEnabled(targetIndex)) {
      setIsNavigating(true);
      setCurrentCarouselIndex(targetIndex);
    }
  };

  const goToNext = () => {
    const targetIndex = currentCarouselIndex + 1;
    if (targetIndex < answers.length && isInputEnabled(targetIndex)) {
      setIsNavigating(true);
      setCurrentCarouselIndex(targetIndex);
    }
  };

  const handleAnswerFocus = (index: number) => {
    if (isKeyboardVisible) {
      setCurrentCarouselIndex(index);
    }
    onAnswerFocus(index);
  };

  const handleAnswerBlur = (index: number) => {
    // Don't handle blur events during navigation to prevent keyboard closure
    if (isNavigating) {
      return;
    }
    
    // Delay blur handling to prevent keyboard closure during transitions
    setTimeout(() => {
      if (!isNavigating) {
        onAnswerBlur(index);
      }
    }, 100);
  };

  const handleEnterPress = (currentIndex: number): void => {
    if (isKeyboardVisible) {
      // In carousel mode, move to next answer
      goToNext();
      return;
    }
    
    const nextIndex = currentIndex + 1;
    
    // If this is the last input and all answers are valid, submit
    if (nextIndex >= answers.length) {
      if (validAnswers.every(valid => valid)) {
        onSubmit();
      }
      return;
    }
    
    // Focus on the next enabled input
    const nextInputEnabled = nextIndex === 0 || validAnswers[nextIndex - 1];
    if (nextInputEnabled && inputRefs.current[nextIndex]) {
      inputRefs.current[nextIndex].focus();
    }
  };
  return (
    <div style={{ marginTop: '20px', width: '100%', boxSizing: 'border-box' }}>
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        marginBottom: '10px',
        width: '100%',
        boxSizing: 'border-box'
      }}>
        <h3 style={{ margin: 0 }}>
          Answers:
          {isWordListLoading && (
            <span style={{ 
              fontSize: '12px', 
              color: '#666', 
              fontWeight: 'normal',
              marginLeft: '8px'
            }}>
              (loading dictionary...)
            </span>
          )}
        </h3>
        <div style={{ 
          fontSize: '18px', 
          fontWeight: 'bold', 
          color: '#4CAF50' 
        }}>
          Total: {scores.reduce((sum, score) => sum + score, 0)}
        </div>
      </div>
      {answers.map((answer, index) => {
        const isValid = validAnswers[index];
        const isEnabled = index === 0 || validAnswers.slice(0, index).every(valid => valid) || isValid;
        const score = scores[index] || 0;
        
        return (
          <AnswerInput
            key={index}
            ref={(el) => { inputRefs.current[index] = el; }}
            index={index}
            value={answer}
            onChange={onAnswerChange}
            isValid={isValid}
            isEnabled={isEnabled}
            score={score}
            onEnterPress={handleEnterPress}
            onFocus={onAnswerFocus}
            onBlur={onAnswerBlur}
            isGameCompleted={isGameCompleted}
          />
        );
      })}
      
      <div style={{ display: 'flex', justifyContent: 'center', marginTop: '20px' }}>
        <button
          onClick={onSubmit}
          disabled={(!validAnswers.every(valid => valid) && !isGameCompleted) || isSubmitting}
          onMouseDown={(e) => {
            const target = e.target as HTMLButtonElement;
            if (!target.disabled) {
              target.style.transform = 'scale(0.9)';
            }
          }}
          onMouseUp={(e) => {
            const target = e.target as HTMLButtonElement;
            if (!target.disabled) {
              target.style.transform = 'scale(1)';
            }
          }}
          onMouseLeave={(e) => {
            const target = e.target as HTMLButtonElement;
            if (!target.disabled) {
              target.style.transform = 'scale(1)';
            }
          }}
          style={{
            padding: '12px 24px',
            fontSize: '16px',
            fontWeight: 'bold',
            backgroundColor: ((validAnswers.every(valid => valid) || isGameCompleted) && !isSubmitting) ? '#4CAF50' : '#cccccc',
            color: ((validAnswers.every(valid => valid) || isGameCompleted) && !isSubmitting) ? 'white' : '#666666',
            border: 'none',
            borderRadius: '8px',
            cursor: ((validAnswers.every(valid => valid) || isGameCompleted) && !isSubmitting) ? 'pointer' : 'not-allowed',
            boxShadow: '0 2px 4px rgba(0,0,0,0.2)',
            transition: 'all 0.1s ease',
            transform: 'scale(1)'
          }}
        >
          {isSubmitting ? 'Submitting...' : (isGameCompleted || isOffline ? 'View Stats' : 'Submit Answers')}
        </button>
      </div>
      
      {/* <ConstraintDisplay 
        board={board}
        answers={answers}
        validAnswers={validAnswers}
        validPaths={validPaths}
      /> */}
    // </div>
  );
}

export default AnswerSection;
