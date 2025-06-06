import React from 'react';
import AnswerInput from './AnswerInput';

function AnswerSection({ 
  answers, 
  onAnswerChange, 
  validAnswers, 
  scores 
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
    </div>
  );
}

export default AnswerSection;