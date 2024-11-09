import React, { useState, useEffect } from 'react';
import { AuthClient } from "@dfinity/auth-client";
import { BingoGame } from './components';

const App = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [authClient, setAuthClient] = useState(null);
  const [principal, setPrincipal] = useState(null);

  useEffect(() => {
    initAuth();
  }, []);

  async function initAuth() {
    const client = await AuthClient.create();
    setAuthClient(client);

    const isAuthenticated = await client.isAuthenticated();
    setIsAuthenticated(isAuthenticated);

    if (isAuthenticated) {
      const identity = client.getIdentity();
      const principal = identity.getPrincipal().toString();
      setPrincipal(principal);
    }
  }

  async function login() {
    const daysToAdd = BigInt(1);
    const EIGHT_HOURS_IN_NANOSECONDS = BigInt(8 * 60 * 60 * 1000000000);
    
    await authClient?.login({
      identityProvider: process.env.II_URL || "https://identity.ic0.app",
      maxTimeToLive: daysToAdd * EIGHT_HOURS_IN_NANOSECONDS,
      onSuccess: async () => {
        setIsAuthenticated(true);
        const identity = authClient.getIdentity();
        const principal = identity.getPrincipal().toString();
        setPrincipal(principal);
      },
    });
  }

  async function logout() {
    await authClient?.logout();
    setIsAuthenticated(false);
    setPrincipal(null);
  }

  return (
    <main className="app-container">
      <h1 className='header'>Bingo Game</h1>
      <div className="auth-container">
        <div className="auth-card">
          <div className="auth-section">
            {isAuthenticated ? (
              <div className="flex-col">
                <div className="auth-info">
                <p className="principal-text">Principal ID: {principal?.slice(0, 8)}...</p>
                <button onClick={logout} className="auth-button logout">
                  Logout
                </button>
              </div>
              <BingoGame />
              </div>
            ) : (
              <button onClick={login} className="auth-button login">
                Login with Internet Identity
              </button>
            )}
          </div>
        </div>
      </div>
      
      <div className="footer">
        <img src="/logo2.svg" alt="DFINITY logo" className="footer-logo" />
      </div>
    </main>
  );
};

export default App;