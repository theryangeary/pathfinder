import React from 'react';
import AnswerInput from './AnswerInput';

function AnswerSection({ 
  answers, 
  onAnswerChange, 
  validAnswers, 
  scores,
  onSubmit
}) {
  return (
    <div style={{ marginTop: '20px' }}>
      <h3>Answers:</h3>
      {answers.map((answer, index) => {
        const isEnabled = index === 0 || validAnswers[index - 1];
        const isValid = validAnswers[index];
        const score = scores[index] || 0;
        
        return (
          <AnswerInput
            key={index}
            index={index}
            value={answer}
            onChange={onAnswerChange}
            isValid={isValid}
            isEnabled={isEnabled}
            score={score}
          />
        );
      })}
      
      <div style={{ marginTop: '20px', fontSize: '18px', fontWeight: 'bold' }}>
        Total Score: {scores.reduce((sum, score) => sum + score, 0)}
      </div>
      
      {validAnswers.every(valid => valid) && (
        <button
          onClick={onSubmit}
          style={{
            marginTop: '20px',
            padding: '12px 24px',
            fontSize: '16px',
            fontWeight: 'bold',
            backgroundColor: '#4CAF50',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer',
            boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
          }}
        >
          Submit Answers
        </button>
      )}
    </div>
  );
}

export default AnswerSection;