import { forwardRef, KeyboardEvent, useImperativeHandle, useRef } from 'react';

interface AnswerInputProps {
  index: number;
  value: string;
  onChange: (index: number, value: string) => void;
  isValid: boolean;
  isEnabled: boolean;
  score: number;
  onEnterPress?: (index: number) => void;
  onFocus?: (index: number) => void;
  onBlur?: (index: number) => void;
  isGameCompleted?: boolean;
}

interface AnswerInputHandle {
  focus: () => void;
}

const AnswerInput = forwardRef<AnswerInputHandle, AnswerInputProps>(function AnswerInput({ 
  index, 
  value, 
  onChange, 
  isValid, 
  isEnabled, 
  score,
  onEnterPress,
  onFocus,
  onBlur,
  isGameCompleted = false
}, ref) {
  const inputRef = useRef<HTMLInputElement>(null);
  const statusIcon = value.length <= 0 ? '' : isValid ? '✅' : '❌';

  useImperativeHandle(ref, () => ({
    focus: () => inputRef.current?.focus()
  }));

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>): void => {
    if (e.key === 'Enter' && isValid && onEnterPress) {
      onEnterPress(index);
    }
  };
  
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
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => isGameCompleted ? undefined : onChange(index, e.target.value.toLowerCase().replace(/[^a-z]/g, ''))}
        onKeyDown={handleKeyDown}
        onFocus={() => onFocus && onFocus(index)}
        onBlur={() => onBlur && onBlur(index)}
        disabled={!isEnabled || isGameCompleted}
        readOnly={isGameCompleted}
        placeholder={`Answer ${index + 1}`}
        style={{
          padding: '8px 12px',
          border: '2px solid #ddd',
          borderRadius: '4px',
          fontSize: '16px',
          minWidth: '200px',
          backgroundColor: isGameCompleted ? '#fcfcfc' : (isEnabled ? '#fff' : '#f5f5f5'),
          cursor: isGameCompleted ? 'default' : 'text'
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
});

export default AnswerInput;
export type { AnswerInputHandle };
