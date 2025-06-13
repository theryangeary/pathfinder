import { useRef } from 'react';
import AnswerInput, { AnswerInputHandle } from './AnswerInput';

interface AnswerSectionProps {
  answers: string[];
  onAnswerChange: (index: number, value: string) => void;
  validAnswers: boolean[];
  scores: number[];
  onSubmit: () => void;
  onAnswerFocus: (index: number) => void;
  isSubmitting?: boolean;
  isWordListLoading?: boolean;
  isGameCompleted?: boolean;
}

function AnswerSection({ 
  answers, 
  onAnswerChange, 
  validAnswers, 
  scores,
  onSubmit,
  onAnswerFocus,
  isSubmitting = false,
  isWordListLoading = false,
  isGameCompleted = false
}: AnswerSectionProps) {
  const inputRefs = useRef<(AnswerInputHandle | null)[]>([]);

  const handleEnterPress = (currentIndex: number): void => {
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
    <div style={{ marginTop: '20px' }}>
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        marginBottom: '10px'
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
          {isSubmitting ? 'Submitting...' : (isGameCompleted ? 'View Stats' : 'Submit Answers')}
        </button>
      </div>
    </div>
  );
}

export default AnswerSection;
