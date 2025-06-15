import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock fetch for API calls
global.fetch = vi.fn()

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
}
global.localStorage = localStorageMock as any

// Mock window.location
delete (window as any).location
window.location = {
  href: 'http://localhost:5173/',
  origin: 'http://localhost:5173',
  pathname: '/',
  search: '',
  hash: '',
} as any