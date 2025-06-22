// Lazy load word list to avoid blocking initial render
import { mergeAllAnswerGroupConstraintSets, UnsatisfiableConstraint } from './constraintResolution';
import { AnswerGroupConstraintSet, Position, Tile } from './models';
import { Answer, findAllPaths, findBestPath } from './pathfinding';
import { scoreAnswerGroup } from './scoring';

  // Validate all answers together - skips invalid words and continues with remaining valid words
export const validateAllAnswers = (board: Tile[][], allAnswers: string[], isValidWordLoaded: boolean, isValidWordFn: ((word: string) => boolean) | null, focusedIndex: number = -1): { validAnswers: boolean[], scores: number[], paths: (Position[] | null)[], constraintSets: AnswerGroupConstraintSet } => {
    const validAnswers = [false, false, false, false, false];
    const scores = [0, 0, 0, 0, 0];
    const paths: (Position[] | null)[] = [null, null, null, null, null];
    const usedWords = new Set<string>();
    
    // Step 1: Sanitize inputs (convert to lowercase)
    const sanitizedAnswers = allAnswers.map(word => word.toLowerCase());
    
    // Step 2: Collect valid words that pass dictionary and path validation
    interface ValidWordInfo {
      index: number;
      originalWord: string;
      sanitizedWord: string;
      answer: Answer;
      isInvalid?: boolean; // Track if this word is invalid (focused input with validation failures)
    }
    
    const validWordsInfo: ValidWordInfo[] = [];

    function passesBasicValidityChecks(sanitizedWord: string): boolean {
      // Skip empty or too short words
      if (!sanitizedWord || sanitizedWord.length < 2) {
        return false;
      }
      
      // Skip if word list is loaded and word is not in dictionary
      if (isValidWordLoaded && isValidWordFn && !isValidWordFn(sanitizedWord)) {
        return false; // Skip this word, don't fail entire validation
      }
      
      return true;
    }
    
    for (let i = 0; i < sanitizedAnswers.length; i++) {
      const originalWord = allAnswers[i];
      const sanitizedWord = sanitizedAnswers[i];
      const isFocused = i === focusedIndex;
      
      // skip certain validation if it is for the current input, to show proper pathfinding
      if (!isFocused && !passesBasicValidityChecks(sanitizedWord)) {
        continue;
      }

      // Check for duplicate words
      if (usedWords.has(sanitizedWord)) {
        continue; // Skip duplicate
      }
      
      // Get all paths for this word using findAllPaths
      const answer = findAllPaths(board, sanitizedWord);
      
      if (answer.paths.length === 0) {
        continue; // Skip this word, don't fail entire validation
      }
      
      // This word is valid - add to our collection
      validWordsInfo.push({
        index: i,
        originalWord,
        sanitizedWord,
        answer
      });
      usedWords.add(sanitizedWord);
    }
    
    // Step 3: Check if the valid words can coexist (constraint compatibility)
    if (validWordsInfo.length === 0) {
      // No valid words at all
      return {
        validAnswers,
        scores,
        paths,
        constraintSets: { pathConstraintSets: [] }
      };
    }
    
    const constraintSets = validWordsInfo.map(info => info.answer.constraintsSet);
    
    let isValidSet = true;
    let finalConstraintSet: AnswerGroupConstraintSet = { pathConstraintSets: [] };
    
    try {
      finalConstraintSet = mergeAllAnswerGroupConstraintSets(constraintSets);
    } catch (e) {
      if (e instanceof UnsatisfiableConstraint) {
        isValidSet = false;
      } else {
        throw e;
      }
    }
    
    if (!isValidSet) {
      // Valid words exist but their constraints conflict - return no valid answers
      return {
        validAnswers,
        scores,
        paths,
        constraintSets: { pathConstraintSets: [] }
      };
    }
    
    // Step 4: All valid words can coexist - populate results
    // Use scoreAnswerGroup to get optimal scores for all words
    const validWords = validWordsInfo.map(info => info.sanitizedWord);
    const { scores: scoresByWord, optimalConstraintSets } = scoreAnswerGroup(validWords, board);
    
    // Use the optimal constraint sets from scoreAnswerGroup instead of manually merging
    finalConstraintSet = { pathConstraintSets: optimalConstraintSets };
    
    for (const info of validWordsInfo) {
      const { index, originalWord, sanitizedWord, isInvalid } = info;
      
      // Find the best path for this word
      const bestPath = findBestPath(board, originalWord, finalConstraintSet);
      if (!bestPath) continue; // Should not happen since we already found paths
      
      validAnswers[index] = true;
      
      // For focused input that fails validation, set score to 0
      scores[index] = isInvalid ? 0 : scoresByWord[sanitizedWord] || 0;
      
      paths[index] = bestPath;
    }

    if (!passesBasicValidityChecks(sanitizedAnswers[focusedIndex])) {
      validAnswers[focusedIndex] = false;
      scores[focusedIndex] = 0;
    }

    return {
      validAnswers,
      scores,
      paths,
      constraintSets: finalConstraintSet
    };
  };
