import React, { useState, useEffect } from 'react';
import Board from './components/Board';
import AnswerSection from './components/AnswerSection';
import HeatmapModal from './components/HeatmapModal';
import { generateBoard } from './utils/boardGeneration';
import { findBestPath, getWildcardConstraintsFromPath, getWildcardAmbiguity, findPathsForHighlighting } from './utils/pathfinding';
import { calculateWordScore } from './utils/scoring';
import { isValidWord } from './data/wordList.js';

function App() {
  const [board, setBoard] = useState([]);
  const [answers, setAnswers] = useState(['', '', '', '', '']);
  const [validAnswers, setValidAnswers] = useState([false, false, false, false, false]);
  const [scores, setScores] = useState([0, 0, 0, 0, 0]);
  const [wildcardConstraints, setWildcardConstraints] = useState({});
  const [highlightedPaths, setHighlightedPaths] = useState([]);
  const [currentInputIndex, setCurrentInputIndex] = useState(-1);
  const [showHeatmapModal, setShowHeatmapModal] = useState(false);
  const [validPaths, setValidPaths] = useState([]);

  useEffect(() => {
    const newBoard = generateBoard();
    setBoard(newBoard);
  }, []);

  const validateAnswer = (word, answerIndex, currentConstraints, previousAnswers = []) => {
    if (!word || word.length < 2) return { isValid: false, score: 0, path: null };
    
    if (!isValidWord(word)) return { isValid: false, score: 0, path: null };
    
    // Check if this word was already used in a previous answer slot
    if (previousAnswers.includes(word.toLowerCase())) {
      return { isValid: false, score: 0, path: null };
    }
    
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

  const handleAnswerFocus = (index) => {
    // Clear highlighting when focusing on a different input
    if (currentInputIndex !== index) {
      setHighlightedPaths([]);
      setCurrentInputIndex(index);
    }
  };

  const handleAnswerChange = (index, value) => {
    const newAnswers = [...answers];
    newAnswers[index] = value;
    setAnswers(newAnswers);

    // Reset constraints and rebuild from scratch based on all valid answers
    let tempConstraints = {};
    const newValidAnswers = [...validAnswers];
    const newScores = [...scores];
    const newValidPaths = [...validPaths];

    // Validate all answers from the beginning, rebuilding constraints as we go
    const validPreviousAnswers = [];
    for (let i = 0; i < 5; i++) {
      if (newAnswers[i]) {
        const result = validateAnswer(newAnswers[i], i, tempConstraints, validPreviousAnswers);
        newValidAnswers[i] = result.isValid;
        newScores[i] = result.score;
        
        if (result.isValid && result.newConstraints) {
          tempConstraints = { ...tempConstraints, ...result.newConstraints };
          // Add this valid answer to the list of previous answers for future validation
          validPreviousAnswers.push(newAnswers[i].toLowerCase());
          // Store the path for heatmap calculation
          newValidPaths[i] = result.path;
        } else {
          newValidPaths[i] = null;
        }

        // Set highlighted paths only for the current input
        if (i === index) {
          const paths = findPathsForHighlighting(board, newAnswers[i], tempConstraints);
          setHighlightedPaths(paths);
          setCurrentInputIndex(index);
        }
      } else {
        // Clear validation state for empty answers
        newValidAnswers[i] = false;
        newScores[i] = 0;
        newValidPaths[i] = null;
        
        // Clear highlighted paths if this is the current input
        if (i === index) {
          setHighlightedPaths([]);
          setCurrentInputIndex(index);
        }
      }
    }

    setValidAnswers(newValidAnswers);
    setScores(newScores);
    setWildcardConstraints(tempConstraints);
    setValidPaths(newValidPaths);

    if (!value) {
      setHighlightedPaths([]);
      setCurrentInputIndex(-1);
    }
  };

  const calculateTileUsage = () => {
    // Initialize 4x4 grid with zeros
    const usage = Array(4).fill().map(() => Array(4).fill(0));
    
    // Count usage from all valid paths
    validPaths.forEach(path => {
      if (path) {
        path.forEach(position => {
          usage[position.row][position.col]++;
        });
      }
    });
    
    return usage;
  };

  const handleSubmit = () => {
    setShowHeatmapModal(true);
  };

  return (
    <div style={{ 
      fontFamily: 'Arial, sans-serif', 
      maxWidth: '800px', 
      margin: '0 auto', 
      padding: '20px',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center'
    }}>
      <h1 style={{ textAlign: 'center' }}>Word Game</h1>
      
      <Board 
        board={board} 
        highlightedPaths={highlightedPaths}
        wildcardConstraints={wildcardConstraints}
        answers={answers}
        validAnswers={validAnswers}
        currentWord={currentInputIndex >= 0 ? answers[currentInputIndex] : ''}
      />
      
      <AnswerSection
        answers={answers}
        onAnswerChange={handleAnswerChange}
        validAnswers={validAnswers}
        scores={scores}
        onSubmit={handleSubmit}
        onAnswerFocus={handleAnswerFocus}
      />
      
      <HeatmapModal
        isOpen={showHeatmapModal}
        onClose={() => setShowHeatmapModal(false)}
        tileUsage={calculateTileUsage()}
        board={board}
        totalScore={scores.reduce((sum, score) => sum + score, 0)}
        scores={scores}
      />
    </div>
  );
}

export default App;