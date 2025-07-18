import { describe, expect, it } from 'vitest'
import {
  convertConstraintSetsToConstraints,
  mergeAllAnswerGroupConstraintSets,
  mergePathConstraintSets,
  UnsatisfiableConstraint
} from '../utils/constraintResolution'
import {
  AnswerGroupConstraintSet,
  PathConstraintSet,
  PathConstraintType,
  Tile
} from '../utils/models'
import { validateAllAnswers } from '../utils/validation'
import { testBoard } from './util.test'

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

// Test case structure for mergePathConstraintSets
interface PathConstraintSetTestCase {
  name: string
  pcs1: PathConstraintSet
  pcs2: PathConstraintSet
  expectError: boolean
  expected?: PathConstraintSet
}

function createPathConstraintSetTestCases(): PathConstraintSetTestCase[] {
  return [
    // === Unconstrained + X cases ===
    {
      name: "Unconstrained + Unconstrained",
      pcs1: unconstrained(),
      pcs2: unconstrained(),
      expectError: false,
      expected: unconstrained()
    },
    {
      name: "Unconstrained + FirstDecided",
      pcs1: unconstrained(),
      pcs2: firstDecided('a'),
      expectError: false,
      expected: firstDecided('a')
    },
    {
      name: "Unconstrained + SecondDecided",
      pcs1: unconstrained(),
      pcs2: secondDecided('b'),
      expectError: false,
      expected: secondDecided('b')
    },
    {
      name: "Unconstrained + BothDecided",
      pcs1: unconstrained(),
      pcs2: bothDecided('a', 'b'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    // === FirstDecided + X cases ===
    {
      name: "FirstDecided + Unconstrained",
      pcs1: firstDecided('a'),
      pcs2: unconstrained(),
      expectError: false,
      expected: firstDecided('a')
    },
    {
      name: "FirstDecided + FirstDecided (same)",
      pcs1: firstDecided('a'),
      pcs2: firstDecided('a'),
      expectError: false,
      expected: firstDecided('a')
    },
    {
      name: "FirstDecided + FirstDecided (different)",
      pcs1: firstDecided('a'),
      pcs2: firstDecided('b'),
      expectError: true
    },
    {
      name: "FirstDecided + SecondDecided",
      pcs1: firstDecided('a'),
      pcs2: secondDecided('b'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "FirstDecided + BothDecided (compatible)",
      pcs1: firstDecided('a'),
      pcs2: bothDecided('a', 'b'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "FirstDecided + BothDecided (incompatible)",
      pcs1: firstDecided('x'),
      pcs2: bothDecided('a', 'b'),
      expectError: true
    },
    // === SecondDecided + X cases ===
    {
      name: "SecondDecided + Unconstrained",
      pcs1: secondDecided('b'),
      pcs2: unconstrained(),
      expectError: false,
      expected: secondDecided('b')
    },
    {
      name: "SecondDecided + FirstDecided",
      pcs1: secondDecided('b'),
      pcs2: firstDecided('a'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "SecondDecided + SecondDecided (same)",
      pcs1: secondDecided('b'),
      pcs2: secondDecided('b'),
      expectError: false,
      expected: secondDecided('b')
    },
    {
      name: "SecondDecided + SecondDecided (different)",
      pcs1: secondDecided('b'),
      pcs2: secondDecided('c'),
      expectError: true
    },
    {
      name: "SecondDecided + BothDecided (compatible)",
      pcs1: secondDecided('b'),
      pcs2: bothDecided('a', 'b'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "SecondDecided + BothDecided (incompatible)",
      pcs1: secondDecided('x'),
      pcs2: bothDecided('a', 'b'),
      expectError: true
    },
    // === BothDecided + X cases ===
    {
      name: "BothDecided + Unconstrained",
      pcs1: bothDecided('a', 'b'),
      pcs2: unconstrained(),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "BothDecided + FirstDecided (compatible)",
      pcs1: bothDecided('a', 'b'),
      pcs2: firstDecided('a'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "BothDecided + FirstDecided (incompatible)",
      pcs1: bothDecided('a', 'b'),
      pcs2: firstDecided('x'),
      expectError: true
    },
    {
      name: "BothDecided + SecondDecided (compatible)",
      pcs1: bothDecided('a', 'b'),
      pcs2: secondDecided('b'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "BothDecided + SecondDecided (incompatible)",
      pcs1: bothDecided('a', 'b'),
      pcs2: secondDecided('x'),
      expectError: true
    },
    {
      name: "BothDecided + BothDecided (same)",
      pcs1: bothDecided('a', 'b'),
      pcs2: bothDecided('a', 'b'),
      expectError: false,
      expected: bothDecided('a', 'b')
    },
    {
      name: "BothDecided + BothDecided (first different)",
      pcs1: bothDecided('a', 'b'),
      pcs2: bothDecided('x', 'b'),
      expectError: true
    },
    {
      name: "BothDecided + BothDecided (second different)",
      pcs1: bothDecided('a', 'b'),
      pcs2: bothDecided('a', 'x'),
      expectError: true
    },
    {
      name: "BothDecided + BothDecided (both different)",
      pcs1: bothDecided('a', 'b'),
      pcs2: bothDecided('x', 'y'),
      expectError: true
    },
    // === Edge cases with same letters ===
    {
      name: "FirstDecided + SecondDecided (same letter)",
      pcs1: firstDecided('a'),
      pcs2: secondDecided('a'),
      expectError: false,
      expected: bothDecided('a', 'a')
    },
    {
      name: "SecondDecided + FirstDecided (same letter)",
      pcs1: secondDecided('a'),
      pcs2: firstDecided('a'),
      expectError: false,
      expected: bothDecided('a', 'a')
    },
    {
      name: "BothDecided same letter both positions",
      pcs1: bothDecided('a', 'a'),
      pcs2: firstDecided('a'),
      expectError: false,
      expected: bothDecided('a', 'a')
    },
    {
      name: "FirstDecided same as BothDecided same letter",
      pcs1: firstDecided('z'),
      pcs2: bothDecided('z', 'z'),
      expectError: false,
      expected: bothDecided('z', 'z')
    },
    {
      name: "SecondDecided same as BothDecided same letter",
      pcs1: secondDecided('z'),
      pcs2: bothDecided('z', 'z'),
      expectError: false,
      expected: bothDecided('z', 'z')
    },
    // === Additional comprehensive coverage ===
    {
      name: "FirstDecided + BothDecided (first matches, different letters)",
      pcs1: firstDecided('x'),
      pcs2: bothDecided('x', 'y'),
      expectError: false,
      expected: bothDecided('x', 'y')
    },
    {
      name: "SecondDecided + BothDecided (second matches, different letters)",
      pcs1: secondDecided('y'),
      pcs2: bothDecided('x', 'y'),
      expectError: false,
      expected: bothDecided('x', 'y')
    },
    {
      name: "BothDecided + FirstDecided (first matches, same letters)",
      pcs1: bothDecided('m', 'm'),
      pcs2: firstDecided('m'),
      expectError: false,
      expected: bothDecided('m', 'm')
    },
    {
      name: "BothDecided + SecondDecided (second matches, same letters)",
      pcs1: bothDecided('n', 'n'),
      pcs2: secondDecided('n'),
      expectError: false,
      expected: bothDecided('n', 'n')
    },
    // === Symmetry tests ===
    {
      name: "Symmetry: FirstDecided('p') + SecondDecided('q')",
      pcs1: firstDecided('p'),
      pcs2: secondDecided('q'),
      expectError: false,
      expected: bothDecided('p', 'q')
    },
    {
      name: "Symmetry: SecondDecided('q') + FirstDecided('p')",
      pcs1: secondDecided('q'),
      pcs2: firstDecided('p'),
      expectError: false,
      expected: bothDecided('p', 'q')
    },
    {
      name: "Symmetry: BothDecided('r', 's') + Unconstrained",
      pcs1: bothDecided('r', 's'),
      pcs2: unconstrained(),
      expectError: false,
      expected: bothDecided('r', 's')
    },
    {
      name: "Symmetry: Unconstrained + BothDecided('r', 's')",
      pcs1: unconstrained(),
      pcs2: bothDecided('r', 's'),
      expectError: false,
      expected: bothDecided('r', 's')
    }
  ]
}

describe('mergePathConstraintSets', () => {
  it('should handle all merge combinations correctly', () => {
    const testCases = createPathConstraintSetTestCases()

    for (const testCase of testCases) {
      if (testCase.expectError) {
        expect(() => {
          mergePathConstraintSets(testCase.pcs1, testCase.pcs2)
        }, `Failed test case: ${testCase.name} - expected error but got success`).toThrow(UnsatisfiableConstraint)
      } else {
        const result = mergePathConstraintSets(testCase.pcs1, testCase.pcs2)

        if (testCase.expected) {
          expect(result, `Failed test case: ${testCase.name}`).toEqual(testCase.expected)
        }
      }
    }
  })

  it('should be commutative for compatible constraints', () => {
    const compatiblePairs = [
      [unconstrained(), firstDecided('a')],
      [firstDecided('a'), secondDecided('b')],
      [secondDecided('b'), unconstrained()],
      [bothDecided('x', 'y'), firstDecided('x')],
      [secondDecided('z'), bothDecided('w', 'z')]
    ]

    for (const [pcs1, pcs2] of compatiblePairs) {
      const result1 = mergePathConstraintSets(pcs1, pcs2)
      const result2 = mergePathConstraintSets(pcs2, pcs1)

      expect(result1).toEqual(result2)
    }
  })

  it('should handle edge cases with same letters', () => {
    // Both wildcards constrained to same letter
    const result1 = mergePathConstraintSets(firstDecided('a'), secondDecided('a'))
    expect(result1).toEqual(bothDecided('a', 'a'))

    // Merge with already-decided same letter constraints
    const result2 = mergePathConstraintSets(bothDecided('x', 'x'), firstDecided('x'))
    expect(result2).toEqual(bothDecided('x', 'x'))

    const result3 = mergePathConstraintSets(secondDecided('y'), bothDecided('y', 'y'))
    expect(result3).toEqual(bothDecided('y', 'y'))
  })

  it('should reject incompatible constraints', () => {
    const incompatiblePairs = [
      [firstDecided('a'), firstDecided('b')],
      [secondDecided('x'), secondDecided('y')],
      [bothDecided('a', 'b'), firstDecided('c')],
      [bothDecided('a', 'b'), secondDecided('c')],
      [bothDecided('a', 'b'), bothDecided('c', 'd')]
    ]

    for (const [pcs1, pcs2] of incompatiblePairs) {
      expect(() => {
        mergePathConstraintSets(pcs1, pcs2)
      }).toThrow(UnsatisfiableConstraint)

      // Should also be commutative for errors
      expect(() => {
        mergePathConstraintSets(pcs2, pcs1)
      }).toThrow(UnsatisfiableConstraint)
    }
  })
})

// Helper function to create test board with the specified layout: 'eadux*ysta*tnhrv'
function createTestBoard(): Tile[][] {
  const layout = 'eadux*ysta*tnhrv'
  const board: Tile[][] = []
  
  for (let row = 0; row < 4; row++) {
    board[row] = []
    for (let col = 0; col < 4; col++) {
      const index = row * 4 + col
      const char = layout[index]
      const isWildcard = char === '*'
      
      board[row][col] = {
        row: row,
        col: col,
        letter: isWildcard ? '' : char.toUpperCase(),
        points: 1, // Simplified for testing
        isWildcard
      }
    }
  }
  
  return board
}

describe('convertConstraintSetsToConstraints', () => {
  const testBoard = createTestBoard()
  
  it('should return empty constraints for empty constraint sets', () => {
    const constraintSets: AnswerGroupConstraintSet = { pathConstraintSets: [] }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({})
  })
  
  it('should return empty constraints when unconstrained path exists', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        unconstrained(),
        firstDecided('A'),
        secondDecided('B')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({})
  })
  
  it('should handle single first wildcard constraint', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [firstDecided('E')]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    // First wildcard is at position (1,1) based on board layout 'eadux*ysta*tnhrv'
    expect(result).toEqual({
      '1-1': 'E'
    })
  })
  
  it('should handle single second wildcard constraint', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [secondDecided('L')]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    // Second wildcard is at position (2,2) based on board layout 'eadux*ysta*tnhrv'
    expect(result).toEqual({
      '2-2': 'L'
    })
  })
  
  it('should handle both wildcards constrained', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [bothDecided('R', 'S')]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '1-1': 'R',
      '2-2': 'S'
    })
  })
  
  it('should handle multiple constraint options with slash notation', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        firstDecided('A'),
        firstDecided('B'),
        bothDecided('A', 'X'),
        bothDecided('B', 'Y')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '1-1': 'A / B',
      '2-2': 'X / Y / *'
    })
  })
  
  it('should handle complex constraint combinations', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        firstDecided('M'),
        secondDecided('N'),
        bothDecided('M', 'O'),
        bothDecided('P', 'N')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '1-1': 'M / P / *',
      '2-2': 'N / O / *'
    })
  })
  
  it('should handle same letter constraints without duplication', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        firstDecided('T'),
        firstDecided('T'),
        bothDecided('T', 'U'),
        bothDecided('T', 'V')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '1-1': 'T',
      '2-2': 'U / V / *'
    })
  })
  
  it('should handle edge case with same letter in both wildcards', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        bothDecided('Z', 'Z'),
        firstDecided('Z'),
        secondDecided('Z')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '1-1': 'Z / *',
      '2-2': 'Z / *'
    })
  })
  
  it('should handle only second wildcard constraints', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        secondDecided('Q'),
        secondDecided('R'),
        secondDecided('S')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '2-2': 'Q / R / S'
    })
  })
  
  it('should handle mixed constraint types with comprehensive coverage', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        firstDecided('A'),
        secondDecided('B'),
        bothDecided('A', 'C'),
        bothDecided('D', 'B'),
        firstDecided('D')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    expect(result).toEqual({
      '1-1': 'A / D / *',
      '2-2': 'B / C / *'
    })
  })
  
  it('should verify wildcard positions for board layout eadux*ysta*tnhrv', () => {
    // Verify that our test board has wildcards in the expected positions
    expect(testBoard[1][1].isWildcard).toBe(true) // position (1,1) - 6th character (0-indexed position 5)
    expect(testBoard[2][2].isWildcard).toBe(true) // position (2,2) - 11th character (0-indexed position 10)
    
    // Verify other positions are not wildcards
    expect(testBoard[0][0].isWildcard).toBe(false) // 'e'
    expect(testBoard[0][1].isWildcard).toBe(false) // 'a'
    expect(testBoard[3][3].isWildcard).toBe(false) // 'v'
  })
  
  it('should handle constraint filtering with proper letter case', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        firstDecided('a'), // lowercase input
        bothDecided('b', 'c'), // lowercase input
        secondDecided('D') // uppercase input
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    // Should convert to uppercase
    expect(result).toEqual({
      '1-1': 'A / B / *',
      '2-2': 'C / D / *'
    })
  })
  
  it('should handle empty letters gracefully', () => {
    const constraintSets: AnswerGroupConstraintSet = {
      pathConstraintSets: [
        { type: PathConstraintType.FirstDecided, firstLetter: undefined },
        { type: PathConstraintType.SecondDecided, secondLetter: '' },
        firstDecided('A')
      ]
    }
    const result = convertConstraintSetsToConstraints(constraintSets, testBoard)
    
    // Should only include valid letters
    expect(result).toEqual({
      '1-1': 'A / *'
    })
  })
})

describe('should constrain correctly', () => {
  it('should resolve year and sev to the same "e"', () => {
    const board = testBoard('eadux*ysta*tnhrv');
    const inputs = ['day', 'year', 'sev', 'data'];
    const validation = validateAllAnswers(board, inputs, true, ((_)=>true))
    expect(validation.validAnswers.slice(0,inputs.length).every((v) => v)).toBeTruthy()
    const constraints = validation.constraintSets
    expect(constraints).toStrictEqual(
      {pathConstraintSets: [
        {type: PathConstraintType.BothDecided, firstLetter: 't', secondLetter: 'e'},
        {type: PathConstraintType.BothDecided, firstLetter: 'a', secondLetter: 'e'}
      ]},
    )
  })
})
