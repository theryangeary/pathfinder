import { useEffect, useRef, useState } from 'react';
import AnswerInput, { AnswerInputHandle } from './AnswerInput';

interface AnswerSectionProps {
  answers: string[];
  onAnswerChange: (index: number, value: string) => void;
  validAnswers: boolean[];
  scores: number[];
  onSubmit: () => void;
  onAnswerFocus: (index: number) => void;
  onAnswerBlur: (index: number) => void;
  onViewAllAnswers: () => void;
  isSubmitting?: boolean;
  isWordListLoading?: boolean;
  isGameCompleted?: boolean;
  shouldUseCompactLayout: boolean;
}

function AnswerSection({ 
  answers, 
  onAnswerChange, 
  validAnswers, 
  scores,
  onSubmit,
  onAnswerFocus,
  onAnswerBlur,
  onViewAllAnswers,
  isSubmitting = false,
  isWordListLoading = false,
  isGameCompleted = false,
  shouldUseCompactLayout,
}: AnswerSectionProps) {
  const inputRefs = useRef<(AnswerInputHandle | null)[]>([]);
  const [currentCarouselIndex, setCurrentCarouselIndex] = useState(0);
  const [isNavigating, setIsNavigating] = useState(false);

  // When compact layout becomes active, find the focused input and set carousel index
  useEffect(() => {
    if (shouldUseCompactLayout) {
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
  }, [shouldUseCompactLayout]);

  // Maintain focus when switching to/from carousel mode or changing carousel index
  useEffect(() => {
    if (shouldUseCompactLayout && inputRefs.current[currentCarouselIndex]) {
      // Use multiple attempts to ensure focus is maintained
      const focusInput = () => {
        inputRefs.current[currentCarouselIndex]?.focus();
        window.scrollTo(0, 0);
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
  }, [shouldUseCompactLayout, currentCarouselIndex]);

  const isInputEnabled = (index: number): boolean => {
    if (index < 0 || index >= answers.length) return false;
    const isValid = validAnswers[index];
    const hasText = Boolean(answers[index] && answers[index].trim().length > 0);
    return index === 0 || validAnswers.slice(0, index).every(valid => valid) || isValid || hasText;
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

  const allAnswersValid = validAnswers.every(valid => valid);
  const isOnLastAnswer = currentCarouselIndex === answers.length - 1;
  const canGoToNext = !isOnLastAnswer && isInputEnabled(currentCarouselIndex + 1);
  const canSubmit = isOnLastAnswer && allAnswersValid;

  const handleSubmit = () => {
    // Blur any focused input to hide the virtual keyboard
    if (document.activeElement && document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
    onSubmit();
  };

  const handleAnswerFocus = (index: number) => {
    setCurrentCarouselIndex(index);
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
    if (shouldUseCompactLayout) {
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
    <div style={
      { marginTop: shouldUseCompactLayout ? '10px' : '20px', 
        width: '100%', 
        boxSizing: 'border-box',
        transition: 'margin 200ms'
      }
    }>
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

          { shouldUseCompactLayout && 
            <div style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              gap: '8px', 
              margin: '0' 
            }}>
              {answers.map((answer, index) => {
                const isValid = validAnswers[index];
                const hasValue = answer.length > 0;
                const isEnabled = isInputEnabled(index);
                const isSelected = index === currentCarouselIndex;
                
                // Determine dot color based on status
                let dotColor = '#666'; // disabled/empty - dark gray
                if (isEnabled) {
                  if (hasValue) {
                    dotColor = isValid ? '#4CAF50' : '#f44336'; // green for valid, red for invalid
                  } else {
                    dotColor = '#ddd'; // gray for empty but enabled
                  }
                }
                
                return (
                  <button
                    key={index}
                    onClick={() => {
                      if (isEnabled) {
                        setIsNavigating(true);
                        setCurrentCarouselIndex(index);
                      }
                    }}
                    onMouseDown={(e) => e.preventDefault()}
                    onTouchStart={(e) => e.preventDefault()}
                    disabled={!isEnabled}
                    style={{
                      width: '12px',
                      height: '12px',
                      borderRadius: '50%',
                      border: isSelected ? '2px solid #333333' : '1px solid rgba(0,0,0,0.1)',
                      backgroundColor: dotColor,
                      cursor: isEnabled ? 'pointer' : 'not-allowed',
                      padding: 0,
                      opacity: isEnabled ? 1 : 0.6,
                      boxSizing: 'border-box'
                    }}
                  />
                );
              })}
            </div>
          }

        <div style={{ 
          fontSize: '18px', 
          fontWeight: 'bold', 
          color: '#4CAF50' 
        }}>
          Total: {scores.reduce((sum, score) => sum + score, 0)}
        </div>
      </div>

          <div style={ shouldUseCompactLayout ?
            { display: 'flex', alignItems: 'center', gap: '10px', margin: '5px 0', width: '100%', boxSizing: 'border-box', transition: '200ms' }
            : {}
          }>
            { shouldUseCompactLayout &&
              <button
                onClick={goToPrevious}
                onMouseDown={(e) => e.preventDefault()}
                onTouchStart={(e) => e.preventDefault()}
                disabled={currentCarouselIndex === 0 || !isInputEnabled(currentCarouselIndex - 1)}
                style={{
                  padding: '8px 8px',
                  fontSize: '18px',
                  backgroundColor: (currentCarouselIndex === 0 || !isInputEnabled(currentCarouselIndex - 1)) ? '#f0f0f0' : '#4CAF50',
                  color: (currentCarouselIndex === 0 || !isInputEnabled(currentCarouselIndex - 1)) ? '#999' : 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: (currentCarouselIndex === 0 || !isInputEnabled(currentCarouselIndex - 1)) ? 'not-allowed' : 'pointer',
                  minWidth: '40px',
                  width: '40px',
                  height: '40px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  flexShrink: 0,
                }}
              >
                ←
              </button>
            }          
            
      {        
      answers.map((answer, index) => {
        const isValid = validAnswers[index];
        const score = scores[index] || 0;
        
        return (
          <AnswerInput
            key={index}
            ref={(el) => { inputRefs.current[index] = el; }}
            index={index}
            value={answer}
            onChange={onAnswerChange}
            isValid={isValid}
            isEnabled={isInputEnabled(index)}
            score={score}
            onEnterPress={handleEnterPress}
            onFocus={handleAnswerFocus}
            onBlur={handleAnswerBlur}
            isGameCompleted={isGameCompleted}
            isVisible={!shouldUseCompactLayout || index === currentCarouselIndex}
            isKeyboardVisible={shouldUseCompactLayout}
          />
        );
      })}
      { shouldUseCompactLayout && 
              <button
                onClick={canSubmit ? handleSubmit : goToNext}
                onMouseDown={(e) => e.preventDefault()}
                onTouchStart={(e) => e.preventDefault()}
                disabled={!canSubmit && !canGoToNext}
                style={{
                  padding: '8px 8px',
                  fontSize: '18px',
                  backgroundColor: (canSubmit || canGoToNext) ? '#4CAF50' : '#f0f0f0',
                  color: (canSubmit || canGoToNext) ? 'white' : '#999',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: (canSubmit || canGoToNext) ? 'pointer' : 'not-allowed',
                  minWidth: '40px',
                  width: '40px',
                  height: '40px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  flexShrink: 0
                }}
              >
                {isOnLastAnswer ? '🚀' : '→'}
              </button>
            }
          </div>
      
      <div style={{ display: 'flex', justifyContent: 'center', marginTop: shouldUseCompactLayout ? '10px' : '20px' }}>
        {shouldUseCompactLayout ? (
          <button
            onClick={onViewAllAnswers}
            onMouseDown={(e) => {
              const target = e.target as HTMLButtonElement;
              target.style.transform = 'scale(0.9)';
            }}
            onMouseUp={(e) => {
              const target = e.target as HTMLButtonElement;
              target.style.transform = 'scale(1)';
            }}
            onMouseLeave={(e) => {
              const target = e.target as HTMLButtonElement;
              target.style.transform = 'scale(1)';
            }}
            style={{
              padding: '6px 24px',
              fontSize: '16px',
              fontWeight: 'bold',
              backgroundColor: '#4CAF50',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              cursor: 'pointer',
              boxShadow: '0 2px 4px rgba(0,0,0,0.2)',
              transition: 'all 0.1s ease',
              transform: 'scale(1)'
            }}
          >
            View All Answers
          </button>
        ) : (
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
            {isSubmitting ? 'Submitting...' : (isGameCompleted ? 'View Stats' : 'Submit Answers')}
          </button>
        )}
      </div>
    </div>
  );
}

export default AnswerSection;
