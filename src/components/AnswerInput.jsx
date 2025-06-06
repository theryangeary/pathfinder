import React from 'react';

function AnswerInput({ 
  index, 
  value, 
  onChange, 
  isValid, 
  isEnabled, 
  score 
}) {
  const statusIcon = isValid ? '✅' : '❌';
  
  return (
    <div 
      style={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: '10px',
        margin: '5px 0',
        opacity: isEnabled ? 1 : 0.5
      }}
    >
      <span style={{ fontSize: '16px', minWidth: '20px' }}>
        {statusIcon}
      </span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(index, e.target.value)}
        disabled={!isEnabled}
        placeholder={`Answer ${index + 1}`}
        style={{
          padding: '8px 12px',
          border: '2px solid #ddd',
          borderRadius: '4px',
          fontSize: '16px',
          minWidth: '200px',
          backgroundColor: isEnabled ? '#fff' : '#f5f5f5'
        }}
      />
      <span 
        style={{ 
          minWidth: '40px', 
          fontWeight: 'bold',
          color: isValid ? '#4caf50' : '#666'
        }}
      >
        {score}
      </span>
    </div>
  );
}

export default AnswerInput;