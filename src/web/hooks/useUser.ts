import { useState, useEffect } from 'react';
import { gameApi } from '../api/gameApi';

const USER_STORAGE_KEY = 'word-game-user';

interface UserSession {
  user_id: string;
  cookie_token: string;
}

export function useUser() {
  const [user, setUser] = useState<UserSession | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Try to get user from localStorage
    const storedUser = localStorage.getItem(USER_STORAGE_KEY);
    if (storedUser) {
      try {
        const parsedUser = JSON.parse(storedUser) as UserSession;
        setUser(parsedUser);
        setIsLoading(false);
        return;
      } catch (error) {
        console.error('Error parsing stored user:', error);
        localStorage.removeItem(USER_STORAGE_KEY);
      }
    }

    // Create new user if none exists
    createNewUser();
  }, []);

  const createNewUser = async () => {
    try {
      setIsLoading(true);
      const newUser = await gameApi.createUser();
      const userSession: UserSession = {
        user_id: newUser.user_id,
        cookie_token: newUser.cookie_token,
      };
      
      setUser(userSession);
      localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(userSession));
    } catch (error) {
      console.error('Error creating user:', error);
      // Create a temporary local user for offline play
      const tempUser: UserSession = {
        user_id: `temp-${Date.now()}`,
        cookie_token: `temp-${Date.now()}`,
      };
      setUser(tempUser);
    } finally {
      setIsLoading(false);
    }
  };

  const clearUser = () => {
    setUser(null);
    localStorage.removeItem(USER_STORAGE_KEY);
    createNewUser();
  };

  return {
    user,
    isLoading,
    createNewUser,
    clearUser,
  };
}