import React, { useRef } from 'react';
import AnswerInput from './AnswerInput';

function AnswerSection({ 
  answers, 
  onAnswerChange, 
  validAnswers, 
  scores,
  onSubmit,
  onAnswerFocus
}) {
  const inputRefs = useRef([]);

  const handleEnterPress = (currentIndex) => {
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
        <h3 style={{ margin: 0 }}>Answers:</h3>
        <div style={{ 
          fontSize: '18px', 
          fontWeight: 'bold', 
          color: '#4CAF50' 
        }}>
          Total: {scores.reduce((sum, score) => sum + score, 0)}
        </div>
      </div>
      {answers.map((answer, index) => {
        const isEnabled = index === 0 || validAnswers[index - 1];
        const isValid = validAnswers[index];
        const score = scores[index] || 0;
        
        return (
          <AnswerInput
            key={index}
            ref={(el) => (inputRefs.current[index] = el)}
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
          disabled={!validAnswers.every(valid => valid)}
          onMouseDown={(e) => {
            if (!e.target.disabled) {
              e.target.style.transform = 'scale(0.9)';
            }
          }}
          onMouseUp={(e) => {
            if (!e.target.disabled) {
              e.target.style.transform = 'scale(1)';
            }
          }}
          onMouseLeave={(e) => {
            if (!e.target.disabled) {
              e.target.style.transform = 'scale(1)';
            }
          }}
          style={{
            padding: '12px 24px',
            fontSize: '16px',
            fontWeight: 'bold',
            backgroundColor: validAnswers.every(valid => valid) ? '#4CAF50' : '#cccccc',
            color: validAnswers.every(valid => valid) ? 'white' : '#666666',
            border: 'none',
            borderRadius: '8px',
            cursor: validAnswers.every(valid => valid) ? 'pointer' : 'not-allowed',
            boxShadow: '0 2px 4px rgba(0,0,0,0.2)',
            transition: 'all 0.1s ease',
            transform: 'scale(1)'
          }}
        >
          Submit Answers
        </button>
      </div>
    </div>
  );
}

export default AnswerSection;
