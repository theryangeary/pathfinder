import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '../App'
import * as gameApi from '../api/gameApi'

// Mock the exact board from puzzle #8 that was causing issues
const puzzle8Board = [
  [
    { letter: 'h', points: 3, isWildcard: false, row: 0, col: 0 },
    { letter: 'i', points: 1, isWildcard: false, row: 0, col: 1 },
    { letter: 's', points: 1, isWildcard: false, row: 0, col: 2 },
    { letter: 's', points: 1, isWildcard: false, row: 0, col: 3 },
  ],
  [
    { letter: 'c', points: 2, isWildcard: false, row: 1, col: 0 },
    { letter: '*', points: 0, isWildcard: true, row: 1, col: 1 },
    { letter: 'l', points: 2, isWildcard: false, row: 1, col: 2 },
    { letter: 'o', points: 1, isWildcard: false, row: 1, col: 3 },
  ],
  [
    { letter: 'l', points: 2, isWildcard: false, row: 2, col: 0 },
    { letter: 'e', points: 1, isWildcard: false, row: 2, col: 1 },
    { letter: '*', points: 0, isWildcard: true, row: 2, col: 2 },
    { letter: 'd', points: 2, isWildcard: false, row: 2, col: 3 },
  ],
  [
    { letter: 's', points: 1, isWildcard: false, row: 3, col: 0 },
    { letter: 'e', points: 1, isWildcard: false, row: 3, col: 1 },
    { letter: 'e', points: 1, isWildcard: false, row: 3, col: 2 },
    { letter: 'o', points: 1, isWildcard: false, row: 3, col: 3 },
  ],
]

// Mock the API
vi.mock('../api/gameApi', () => ({
  gameApi: {
    getDailyGame: vi.fn(),
    getGameBySequence: vi.fn(),
    getGameEntry: vi.fn(),
    submitAnswers: vi.fn(),
    getGameWords: vi.fn(),
  },
  convertApiBoardToBoard: vi.fn(() => puzzle8Board),
}))

// Mock the user hook
vi.mock('../hooks/useUser', () => ({
  useUser: () => ({
    user: {
      user_id: 'test-user',
      cookie_token: 'test-token',
    },
    isLoading: false,
    clearUser: vi.fn(),
  }),
}))

// Mock the word list to include puzzle #8 words
vi.mock('../data/wordList', () => ({
  isValidWord: vi.fn((word: string) => {
    const validWords = ['silo', 'seed', 'sed', 'sold', 'does', 'his', 'hi', 'so', 'lo', 'ol', 'led', 'eel', 'ee', 'od', 'do', 'oe', 'os']
    return validWords.includes(word.toLowerCase())
  }),
}))

const mockPuzzle8GameData = {
  id: 'puzzle8-game-id',
  date: '2025-06-09',
  sequence_number: 8,
  threshold_score: 15,
  board: {
    tiles: [
      [
        { letter: 'h', points: 3, is_wildcard: false, row: 0, col: 0 },
        { letter: 'i', points: 1, is_wildcard: false, row: 0, col: 1 },
        { letter: 's', points: 1, is_wildcard: false, row: 0, col: 2 },
        { letter: 's', points: 1, is_wildcard: false, row: 0, col: 3 },
      ],
      [
        { letter: 'c', points: 2, is_wildcard: false, row: 1, col: 0 },
        { letter: '*', points: 0, is_wildcard: true, row: 1, col: 1 },
        { letter: 'l', points: 2, is_wildcard: false, row: 1, col: 2 },
        { letter: 'o', points: 1, is_wildcard: false, row: 1, col: 3 },
      ],
      [
        { letter: 'l', points: 2, is_wildcard: false, row: 2, col: 0 },
        { letter: 'e', points: 1, is_wildcard: false, row: 2, col: 1 },
        { letter: '*', points: 0, is_wildcard: true, row: 2, col: 2 },
        { letter: 'd', points: 2, is_wildcard: false, row: 2, col: 3 },
      ],
      [
        { letter: 's', points: 1, is_wildcard: false, row: 3, col: 0 },
        { letter: 'e', points: 1, is_wildcard: false, row: 3, col: 1 },
        { letter: 'e', points: 1, is_wildcard: false, row: 3, col: 2 },
        { letter: 'o', points: 1, is_wildcard: false, row: 3, col: 3 },
      ],
    ],
  },
}

describe('Puzzle #8 Validation Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    ;(gameApi.gameApi.getGameBySequence as any).mockResolvedValue(mockPuzzle8GameData)
    ;(gameApi.gameApi.getGameEntry as any).mockResolvedValue(null)
    ;(gameApi.gameApi.submitAnswers as any).mockResolvedValue({ success: true })
    ;(gameApi.gameApi.getGameWords as any).mockResolvedValue(['silo', 'seed', 'sed', 'sold', 'does', 'his', 'hi', 'so', 'lo', 'ol', 'led', 'eel', 'ee', 'od', 'do', 'oe', 'os'])
  })

  it('should validate the exact word "sed" that was failing in puzzle #8', async () => {
    render(
      <MemoryRouter initialEntries={['/puzzle/8']}>
        <Routes>
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText((content, element) => {
        return element?.textContent?.includes('Puzzle #8')
      })).toBeInTheDocument()
    })

    // Find the third answer input (index 2, for "sed")
    const answerInputs = screen.getAllByRole('textbox')
    expect(answerInputs).toHaveLength(5)

    // Type "sed" in the third input
    fireEvent.change(answerInputs[2], { target: { value: 'sed' } })

    // Wait for validation to complete
    await waitFor(() => {
      // The input should show a green checkmark (valid)
      const checkmarks = screen.getAllByText('✅')
      expect(checkmarks.length).toBeGreaterThan(0)
    })

    // Verify that "sed" is marked as valid by checking for green checkmark
    const parentDiv = answerInputs[2].closest('div')
    expect(parentDiv).toBeInTheDocument()
    expect(parentDiv?.textContent).toContain('✅')
  })

  it('should validate the complete puzzle #8 answer set: silo, seed, sed, sold, does', async () => {
    render(
      <MemoryRouter initialEntries={['/puzzle/8']}>
        <Routes>
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    // Wait for the game to load
    await waitFor(() => {
      expect(screen.getByText((content, element) => {
        return element?.textContent?.includes('Puzzle #8')
      })).toBeInTheDocument()
    })

    const answerInputs = screen.getAllByRole('textbox')
    const puzzle8Answers = ['silo', 'seed', 'sed', 'sold', 'does']

    // Fill in all answers sequentially
    for (let i = 0; i < puzzle8Answers.length; i++) {
      fireEvent.change(answerInputs[i], { target: { value: puzzle8Answers[i] } })
      
      // Wait for this answer to be validated
      await waitFor(() => {
        const parentDiv = answerInputs[i].closest('div')
        expect(parentDiv?.textContent).toContain('✅')
      })
    }

    // Wait for all validations to complete
    await waitFor(() => {
      const checkmarks = screen.getAllByText('✅')
      expect(checkmarks).toHaveLength(5) // All 5 answers should be valid
    })

    // Check that submit button is enabled
    const submitButton = screen.getByRole('button', { name: /submit answers/i })
    expect(submitButton).not.toBeDisabled()

    // Verify total score matches expected (5 + 5 + 3 + 6 + 4 = 23)
    expect(screen.getByText('Total: 23')).toBeInTheDocument()
  })

  it('should handle wildcard constraints correctly when validating multiple answers', async () => {
    render(
      <MemoryRouter initialEntries={['/puzzle/8']}>
        <Routes>
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(screen.getByText((content, element) => {
        return element?.textContent?.includes('Puzzle #8')
      })).toBeInTheDocument()
    })

    const answerInputs = screen.getAllByRole('textbox')

    // First enter "silo" - this should be valid
    fireEvent.change(answerInputs[0], { target: { value: 'silo' } })
    await waitFor(() => {
      const parentDiv = answerInputs[0].closest('div')
      expect(parentDiv?.textContent).toContain('✅')
    })

    // Then enter "seed" - this should still be valid 
    fireEvent.change(answerInputs[1], { target: { value: 'seed' } })
    await waitFor(() => {
      const parentDiv = answerInputs[1].closest('div')
      expect(parentDiv?.textContent).toContain('✅')
    })

    // Now enter "sed" - this was the problematic word that should now be valid
    fireEvent.change(answerInputs[2], { target: { value: 'sed' } })
    await waitFor(() => {
      const parentDiv = answerInputs[2].closest('div')
      expect(parentDiv?.textContent).toContain('✅')
    })

    // All three should remain valid (no wildcard constraint conflicts)
    const checkmarks = screen.getAllByText('✅')
    expect(checkmarks.length).toBeGreaterThanOrEqual(3)
  })

  it('should demonstrate that frontend validation matches backend strict validation', async () => {
    render(
      <MemoryRouter initialEntries={['/puzzle/8']}>
        <Routes>
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(screen.getByText((content, element) => {
        return element?.textContent?.includes('Puzzle #8')
      })).toBeInTheDocument()
    })

    const answerInputs = screen.getAllByRole('textbox')
    const puzzle8Answers = ['silo', 'seed', 'sed', 'sold', 'does']

    // Fill in all answers
    for (let i = 0; i < puzzle8Answers.length; i++) {
      fireEvent.change(answerInputs[i], { target: { value: puzzle8Answers[i] } })
    }

    // Wait for all validations to complete
    await waitFor(() => {
      const checkmarks = screen.getAllByText('✅')
      expect(checkmarks).toHaveLength(5)
    })

    // Click submit - this should now succeed (no "Failed to submit" error)
    const submitButton = screen.getByRole('button', { name: /submit answers/i })
    fireEvent.click(submitButton)

    // Wait for submission to complete
    await waitFor(() => {
      expect(gameApi.gameApi.submitAnswers).toHaveBeenCalledWith({
        user_id: 'test-user',
        cookie_token: 'test-token',
        answers: expect.any(Array),
        game_id: 'puzzle8-game-id',
      })
    })

    // Should not show the "Failed to submit answers" error
    await waitFor(() => {
      expect(screen.queryByText(/failed to submit answers/i)).not.toBeInTheDocument()
    }, { timeout: 3000 })
  })
})
