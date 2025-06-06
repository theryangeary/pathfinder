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
            index={index}
            value={answer}
            onChange={onAnswerChange}
            isValid={isValid}
            isEnabled={isEnabled}
            score={score}
          />
        );
      })}
      
      <div style={{ display: 'flex', justifyContent: 'center', marginTop: '20px' }}>
        <button
          onClick={onSubmit}
          disabled={!validAnswers.every(valid => valid)}
          style={{
            padding: '12px 24px',
            fontSize: '16px',
            fontWeight: 'bold',
            backgroundColor: validAnswers.every(valid => valid) ? '#4CAF50' : '#cccccc',
            color: validAnswers.every(valid => valid) ? 'white' : '#666666',
            border: 'none',
            borderRadius: '8px',
            cursor: validAnswers.every(valid => valid) ? 'pointer' : 'not-allowed',
            boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
          }}
        >
          Submit Answers
        </button>
      </div>
    </div>
  );
}

export default AnswerSection;