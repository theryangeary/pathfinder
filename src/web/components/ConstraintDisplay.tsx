import { Position, Tile } from '../utils/scoring';
import { 
  getAnswerGroupConstraintSets, 
  mergeAllAnswerGroupConstraintSets, 
  formatConstraintSet,
  UnsatisfiableConstraint 
} from '../utils/constraintResolution';

interface ConstraintDisplayProps {
  board: Tile[][];
  answers: string[];
  validAnswers: boolean[];
  validPaths: (Position[] | null)[];
}

function ConstraintDisplay({ board, answers, validAnswers, validPaths }: ConstraintDisplayProps) {
  // Only show if there are valid answers
  const hasValidAnswers = validAnswers.some(valid => valid);
  
  if (!hasValidAnswers) {
    return null;
  }
  
  try {
    // Get constraint sets for all valid answers
    const answerConstraintSets = getAnswerGroupConstraintSets(board, answers, validAnswers, validPaths);
    
    if (answerConstraintSets.length === 0) {
      return (
        <div style={{ 
          marginTop: '15px', 
          padding: '10px', 
          backgroundColor: '#f5f5f5', 
          borderRadius: '5px',
          fontSize: '12px',
          color: '#666'
        }}>
          <strong>Constraint Resolution:</strong> No constraints to resolve
        </div>
      );
    }
    
    // Merge all constraint sets
    const mergedConstraints = mergeAllAnswerGroupConstraintSets(answerConstraintSets);
    const formattedResult = formatConstraintSet(mergedConstraints);
    
    return (
      <div style={{ 
        marginTop: '15px', 
        padding: '10px', 
        backgroundColor: '#f5f5f5', 
        borderRadius: '5px',
        fontSize: '12px',
        color: '#333'
      }}>
        <strong>Constraint Resolution:</strong>
        <div style={{ 
          marginTop: '5px', 
          whiteSpace: 'pre-line',
          fontFamily: 'monospace'
        }}>
          {formattedResult}
        </div>
      </div>
    );
    
  } catch (error) {
    if (error instanceof UnsatisfiableConstraint) {
      return (
        <div style={{ 
          marginTop: '15px', 
          padding: '10px', 
          backgroundColor: '#ffebee', 
          borderRadius: '5px',
          fontSize: '12px',
          color: '#c62828'
        }}>
          <strong>Constraint Resolution:</strong> Unsatisfiable constraints detected
        </div>
      );
    }
    
    return (
      <div style={{ 
        marginTop: '15px', 
        padding: '10px', 
        backgroundColor: '#fff3e0', 
        borderRadius: '5px',
        fontSize: '12px',
        color: '#ef6c00'
      }}>
        <strong>Constraint Resolution:</strong> Error: {error instanceof Error ? error.message : 'Unknown error'}
      </div>
    );
  }
}

export default ConstraintDisplay;