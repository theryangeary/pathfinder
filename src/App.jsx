import React, { useState, useEffect } from 'react';
import Board from './components/Board';
import AnswerSection from './components/AnswerSection';
import { generateBoard } from './utils/boardGeneration';
import { findBestPath, getWildcardConstraintsFromPath } from './utils/pathfinding';
import { calculateWordScore } from './utils/scoring';
import { isValidWord } from './data/wordList';

function App() {
  const [board, setBoard] = useState([]);
  const [answers, setAnswers] = useState(['', '', '', '', '']);
  const [validAnswers, setValidAnswers] = useState([false, false, false, false, false]);
  const [scores, setScores] = useState([0, 0, 0, 0, 0]);
  const [wildcardConstraints, setWildcardConstraints] = useState({});
  const [highlightedPath, setHighlightedPath] = useState([]);
  const [currentInputIndex, setCurrentInputIndex] = useState(-1);

  useEffect(() => {
    const newBoard = generateBoard();
    setBoard(newBoard);
  }, []);

  const validateAnswer = (word, answerIndex, currentConstraints) => {
    if (!word || word.length < 2) return { isValid: false, score: 0, path: null };
    
    if (!isValidWord(word)) return { isValid: false, score: 0, path: null };
    
    const path = findBestPath(board, word, currentConstraints);
    if (!path) return { isValid: false, score: 0, path: null };
    
    const newConstraints = getWildcardConstraintsFromPath(board, word, path);
    
    for (const [key, value] of Object.entries(newConstraints)) {
      if (currentConstraints[key] && currentConstraints[key] !== value) {
        return { isValid: false, score: 0, path: null };
      }
    }
    
    const score = calculateWordScore(word, path, board);
    return { isValid: true, score, path, newConstraints };
  };

  const handleAnswerChange = (index, value) => {
    const newAnswers = [...answers];
    newAnswers[index] = value;
    setAnswers(newAnswers);

    let tempConstraints = { ...wildcardConstraints };
    const newValidAnswers = [...validAnswers];
    const newScores = [...scores];

    for (let i = 0; i <= index; i++) {
      if (i < index) {
        continue;
      }
      
      const result = validateAnswer(newAnswers[i], i, tempConstraints);
      newValidAnswers[i] = result.isValid;
      newScores[i] = result.score;
      
      if (result.isValid && result.newConstraints) {
        tempConstraints = { ...tempConstraints, ...result.newConstraints };
      }

      if (i === index && result.path) {
        setHighlightedPath(result.path);
        setCurrentInputIndex(index);
      }
    }

    for (let i = index + 1; i < 5; i++) {
      if (newAnswers[i]) {
        const result = validateAnswer(newAnswers[i], i, tempConstraints);
        newValidAnswers[i] = result.isValid;
        newScores[i] = result.score;
        
        if (result.isValid && result.newConstraints) {
          tempConstraints = { ...tempConstraints, ...result.newConstraints };
        }
      }
    }

    setValidAnswers(newValidAnswers);
    setScores(newScores);
    setWildcardConstraints(tempConstraints);

    if (!value) {
      setHighlightedPath([]);
      setCurrentInputIndex(-1);
    }
  };

  return (
    <div style={{ 
      fontFamily: 'Arial, sans-serif', 
      maxWidth: '800px', 
      margin: '0 auto', 
      padding: '20px' 
    }}>
      <h1 style={{ textAlign: 'center' }}>Word Game</h1>
      
      <Board 
        board={board} 
        highlightedPath={highlightedPath}
        wildcardConstraints={wildcardConstraints}
      />
      
      <AnswerSection
        answers={answers}
        onAnswerChange={handleAnswerChange}
        validAnswers={validAnswers}
        scores={scores}
      />
    </div>
  );
}

export default App;