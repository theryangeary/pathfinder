import { describe, expect, it } from 'vitest'
import { 
  mergeAllAnswerGroupConstraintSets,
  PathConstraintType,
  PathConstraintSet,
  AnswerGroupConstraintSet,
  UnsatisfiableConstraint
} from '../utils/constraintResolution'

// Helper function to create AnswerGroupConstraintSet from PathConstraintSets
function answerGroupFrom(pathConstraintSets: PathConstraintSet[]): AnswerGroupConstraintSet {
  return { pathConstraintSets }
}

// Helper function to create PathConstraintSet variants
function unconstrained(): PathConstraintSet {
  return { type: PathConstraintType.Unconstrained }
}

function firstDecided(letter: string): PathConstraintSet {
  return { type: PathConstraintType.FirstDecided, firstLetter: letter }
}

function secondDecided(letter: string): PathConstraintSet {
  return { type: PathConstraintType.SecondDecided, secondLetter: letter }
}

function bothDecided(firstLetter: string, secondLetter: string): PathConstraintSet {
  return { type: PathConstraintType.BothDecided, firstLetter, secondLetter }
}

// Test case structure
interface MergeAllTestCase {
  name: string
  inputSets: AnswerGroupConstraintSet[]
  expectedError: boolean
  expectedResult?: AnswerGroupConstraintSet
}

function createMergeAllTestCases(): MergeAllTestCase[] {
  return [
    // Empty input cases
    {
      name: "Empty input vector",
      inputSets: [],
      expectedError: true
    },
    {
      name: "Single empty set",
      inputSets: [answerGroupFrom([])],
      expectedError: false,
      expectedResult: answerGroupFrom([])
    },
    {
      name: "Two empty sets",
      inputSets: [answerGroupFrom([]), answerGroupFrom([])],
      expectedError: true
    },
    
    // Single set cases
    {
      name: "Single non-empty set",
      inputSets: [answerGroupFrom([unconstrained(), firstDecided('a')])],
      expectedError: false,
      expectedResult: answerGroupFrom([unconstrained(), firstDecided('a')])
    },
    
    // Two set cases - compatible
    {
      name: "Two compatible sets",
      inputSets: [
        answerGroupFrom([unconstrained(), firstDecided('a')]),
        answerGroupFrom([secondDecided('b'), unconstrained()])
      ],
      expectedError: false,
      expectedResult: answerGroupFrom([
        secondDecided('b'),
        unconstrained(),
        bothDecided('a', 'b'),
        firstDecided('a')
      ])
    },
    
    // Two set cases - incompatible
    {
      name: "Two completely incompatible sets",
      inputSets: [
        answerGroupFrom([firstDecided('a')]),
        answerGroupFrom([firstDecided('b')])
      ],
      expectedError: true
    },
    
    // Two set cases - partially compatible  
    {
      name: "Two partially compatible sets",
      inputSets: [
        answerGroupFrom([firstDecided('a'), unconstrained()]),
        answerGroupFrom([firstDecided('b'), firstDecided('a')])
      ],
      expectedError: false,
      expectedResult: answerGroupFrom([firstDecided('a'), firstDecided('b'), firstDecided('a')])
    },
    
    // Three set cases - all compatible
    {
      name: "Three compatible sets",
      inputSets: [
        answerGroupFrom([unconstrained()]),
        answerGroupFrom([firstDecided('x')]),
        answerGroupFrom([secondDecided('y')])
      ],
      expectedError: false,
      expectedResult: answerGroupFrom([bothDecided('x', 'y')])
    },
    
    // Three set cases - incompatible
    {
      name: "Three sets with conflict",
      inputSets: [
        answerGroupFrom([firstDecided('a')]),
        answerGroupFrom([firstDecided('a')]),
        answerGroupFrom([firstDecided('b')])
      ],
      expectedError: true
    },
    
    // Complex cases
    {
      name: "Multiple constraint sets with intersections",
      inputSets: [
        answerGroupFrom([unconstrained(), firstDecided('a'), secondDecided('b')]),
        answerGroupFrom([firstDecided('a'), bothDecided('a', 'c')]),
        answerGroupFrom([secondDecided('b')])
      ],
      expectedError: false,
      expectedResult: answerGroupFrom([bothDecided('a', 'b')])
    },
    
    // Edge cases
    {
      name: "Duplicate constraints",
      inputSets: [
        answerGroupFrom([firstDecided('x'), firstDecided('x')]),
        answerGroupFrom([firstDecided('x')])
      ],
      expectedError: false,
      expectedResult: answerGroupFrom([firstDecided('x')])
    },
    
    {
      name: "Gradual constraint tightening",
      inputSets: [
        answerGroupFrom([unconstrained(), firstDecided('a')]),
        answerGroupFrom([unconstrained(), secondDecided('b')]),
        answerGroupFrom([bothDecided('a', 'b')])
      ],
      expectedError: false,
      expectedResult: answerGroupFrom([bothDecided('a', 'b')])
    }
  ]
}

describe('mergeAllAnswerGroupConstraintSets', () => {
  it('should handle all test cases correctly', () => {
    const testCases = createMergeAllTestCases()
    
    for (const testCase of testCases) {
      if (testCase.expectedError) {
        expect(() => {
          mergeAllAnswerGroupConstraintSets(testCase.inputSets)
        }, `Failed test case: ${testCase.name} - expected error but got success`).toThrow(UnsatisfiableConstraint)
      } else {
        const result = mergeAllAnswerGroupConstraintSets(testCase.inputSets)
        
        if (testCase.expectedResult) {
          // Compare as sets since order doesn't matter
          const actualSet = new Set(result.pathConstraintSets.map(pcs => JSON.stringify(pcs)))
          const expectedSet = new Set(testCase.expectedResult.pathConstraintSets.map(pcs => JSON.stringify(pcs)))
          
          expect(actualSet, `Failed test case: ${testCase.name}`).toEqual(expectedSet)
        }
      }
    }
  })
  
  // Additional specific test cases for edge scenarios
  it('should handle empty input correctly', () => {
    expect(() => {
      mergeAllAnswerGroupConstraintSets([])
    }).toThrow(UnsatisfiableConstraint)
  })
  
  it('should handle single set correctly', () => {
    const singleSet = answerGroupFrom([unconstrained(), firstDecided('a')])
    const result = mergeAllAnswerGroupConstraintSets([singleSet])
    
    expect(result).toEqual(singleSet)
  })
  
  it('should preserve constraint relationships', () => {
    const sets = [
      answerGroupFrom([unconstrained(), firstDecided('a')]),
      answerGroupFrom([secondDecided('b'), unconstrained()])
    ]
    
    const result = mergeAllAnswerGroupConstraintSets(sets)
    
    // Should contain all possible constraint combinations
    const resultTypes = new Set(result.pathConstraintSets.map(pcs => pcs.type))
    expect(resultTypes).toContain(PathConstraintType.Unconstrained)
    expect(resultTypes).toContain(PathConstraintType.FirstDecided)
    expect(resultTypes).toContain(PathConstraintType.SecondDecided)
    expect(resultTypes).toContain(PathConstraintType.BothDecided)
  })
})