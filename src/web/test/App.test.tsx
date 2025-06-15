import { describe, it, expect, vi, beforeEach, Mock } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { MemoryRouter } from 'react-router-dom'
import App from '../App'
import * as gameApi from '../api/gameApi'

// Mock the API
vi.mock('../api/gameApi', () => ({
  gameApi: {
    getDailyGame: vi.fn(),
    getGameBySequence: vi.fn(),
    getGameEntry: vi.fn(),
    getGameWords: vi.fn(),
  },
  convertApiBoardToBoard: vi.fn(() => [
    [
      { letter: 't', points: 1, isWildcard: false, row: 0, col: 0 },
      { letter: 'e', points: 1, isWildcard: false, row: 0, col: 1 },
      { letter: 's', points: 1, isWildcard: false, row: 0, col: 2 },
      { letter: 't', points: 1, isWildcard: false, row: 0, col: 3 },
    ],
    [
      { letter: 'w', points: 3, isWildcard: false, row: 1, col: 0 },
      { letter: 'o', points: 1, isWildcard: false, row: 1, col: 1 },
      { letter: 'r', points: 1, isWildcard: false, row: 1, col: 2 },
      { letter: 'd', points: 2, isWildcard: false, row: 1, col: 3 },
    ],
    [
      { letter: 'h', points: 3, isWildcard: false, row: 2, col: 0 },
      { letter: 'e', points: 1, isWildcard: false, row: 2, col: 1 },
      { letter: 'l', points: 2, isWildcard: false, row: 2, col: 2 },
      { letter: 'l', points: 2, isWildcard: false, row: 2, col: 3 },
    ],
    [
      { letter: 'o', points: 1, isWildcard: false, row: 3, col: 0 },
      { letter: '*', points: 0, isWildcard: true, row: 3, col: 1 },
      { letter: '*', points: 0, isWildcard: true, row: 3, col: 2 },
      { letter: 'o', points: 1, isWildcard: false, row: 3, col: 3 },
    ],
  ]),
}))

// Mock the user hook
vi.mock('../hooks/useUser', () => ({
  useUser: () => ({
    user: null,
    isLoading: false,
    clearUser: vi.fn(),
  }),
}))

// Mock the word list
vi.mock('../data/wordList', () => ({
  isValidWord: vi.fn(() => true),
}))

const mockGameData = {
  id: 'test-game-id',
  date: '2025-06-08',
  sequence_number: 1,
  threshold_score: 40,
  board: {
    tiles: [
      [
        { letter: 't', points: 1, is_wildcard: false, row: 0, col: 0 },
        { letter: 'e', points: 1, is_wildcard: false, row: 0, col: 1 },
        { letter: 's', points: 1, is_wildcard: false, row: 0, col: 2 },
        { letter: 't', points: 1, is_wildcard: false, row: 0, col: 3 },
      ],
    ],
  },
}

const mockSequenceGameData = {
  ...mockGameData,
  id: 'test-sequence-game-id',
  sequence_number: 5,
  date: '2025-06-07',
}

describe('App Component Routing', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    ;(gameApi.gameApi.getDailyGame as Mock).mockResolvedValue(mockGameData)
    ;(gameApi.gameApi.getGameBySequence as Mock).mockResolvedValue(mockSequenceGameData)
    ;(gameApi.gameApi.getGameEntry as Mock).mockResolvedValue(null)
    ;(gameApi.gameApi.getGameWords as Mock).mockResolvedValue(['test', 'word', 'list'])
  })

  it('should load daily game when accessing root path', async () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(gameApi.gameApi.getDailyGame).toHaveBeenCalledTimes(1)
      expect(gameApi.gameApi.getGameBySequence).not.toHaveBeenCalled()
    })

    await waitFor(() => {
      expect(screen.getAllByText((content, element) => {
        return element?.textContent === 'Puzzle #1 · 2025-06-08'
      })[0]).toBeInTheDocument()
    })
  })

  it('should load specific game when accessing puzzle sequence path', async () => {
    render(
      <MemoryRouter initialEntries={['/puzzle/5']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(gameApi.gameApi.getGameBySequence).toHaveBeenCalledWith(5)
      expect(gameApi.gameApi.getDailyGame).not.toHaveBeenCalled()
    })

    await waitFor(() => {
      expect(screen.getByText((content, element) => {
        return element?.textContent?.includes('Puzzle #5')
      })).toBeInTheDocument()
    })
  })

  it('should load different sequence numbers correctly', async () => {
    const { rerender } = render(
      <MemoryRouter initialEntries={['/puzzle/1']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(gameApi.gameApi.getGameBySequence).toHaveBeenCalledWith(1)
    })

    // Simulate navigation to different sequence number
    rerender(
      <MemoryRouter initialEntries={['/puzzle/10']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(gameApi.gameApi.getGameBySequence).toHaveBeenCalledWith(10)
    })
  })

  it('should handle invalid sequence number parameters', async () => {
    ;(gameApi.gameApi.getGameBySequence as Mock).mockRejectedValue(new Error('Not found'))

    render(
      <MemoryRouter initialEntries={['/puzzle/999']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(gameApi.gameApi.getGameBySequence).toHaveBeenCalledWith(999)
    })

    // Should show error state or fallback to local generation
    await waitFor(() => {
      expect(screen.getByText(/failed to connect to server/i)).toBeInTheDocument()
    })
  })

  it('should parse sequence number from URL correctly', async () => {
    const testCases = [
      { path: '/puzzle/1', expectedSequence: 1 },
      { path: '/puzzle/42', expectedSequence: 42 },
      { path: '/puzzle/999', expectedSequence: 999 },
    ]

    for (const testCase of testCases) {
      render(
        <MemoryRouter initialEntries={[testCase.path]}>
          <Routes>
            <Route path="/" element={<App />} />
            <Route path="/puzzle/:sequenceNumber" element={<App />} />
          </Routes>
        </MemoryRouter>
      )

      await waitFor(() => {
        expect(gameApi.gameApi.getGameBySequence).toHaveBeenCalledWith(testCase.expectedSequence)
      })

      vi.clearAllMocks()
    }
  })

  it('should display correct sequence number in UI', async () => {
    render(
      <MemoryRouter initialEntries={['/puzzle/5']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(screen.getByText((content, element) => {
        return element?.textContent?.includes('Puzzle #5')
      })).toBeInTheDocument()
    })
  })

  it('should handle API errors gracefully and show fallback', async () => {
    ;(gameApi.gameApi.getGameBySequence as Mock).mockRejectedValue(new Error('API Error'))

    render(
      <MemoryRouter initialEntries={['/puzzle/1']}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/puzzle/:sequenceNumber" element={<App />} />
        </Routes>
      </MemoryRouter>
    )

    await waitFor(() => {
      expect(gameApi.gameApi.getGameBySequence).toHaveBeenCalledWith(1)
    })

    // Should show offline/fallback message
    await waitFor(() => {
      expect(screen.getByText(/failed to connect to server/i)).toBeInTheDocument()
    })
  })
})