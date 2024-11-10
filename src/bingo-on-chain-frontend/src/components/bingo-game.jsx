import React, { useState, useEffect } from "react";
import "./bingo-game.scss";
import { bingo_on_chain_backend } from "declarations/bingo-on-chain-backend";

const BingoGame = () => {
  const HEADERS = ["B", "I", "N", "G", "O"];
  const CALL_INTERVAL = 15; // seconds

  // Add timer state
  const [timeUntilNextCall, setTimeUntilNextCall] = useState(CALL_INTERVAL);

  // Your existing states
  const [board, setBoard] = useState(Array(25).fill(null));
  const [clicked, setClicked] = useState(Array(25).fill(false));
  const [canChallenge, setCanChallenge] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const [challengeResult, setChallengeResult] = useState(null);
  const [calledNumber, setCalledNumber] = useState(null);
  const [recentCalls, setRecentCalls] = useState([]);
  const [gameIsActive, setGameIsActive] = useState(false);

  // Initialize board with mock data
  useEffect(() => {
    generateCardNumber();
  }, []);

  // Format time for display
  const formatTime = (seconds) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  //generate card 
  const generateCardNumber = async () => {
    try {
      const [card, error] = await bingo_on_chain_backend.generate_card();
      
      if (error) {
        console.log(error);
        return;
      }
      if (card) {
        return;
      } else {
        alert('No card was generated');
      }
    } catch (error) {
      console.log(error);
    }
  };

  const getCard = async () => {
    try {
      bingo_on_chain_backend.get_card().then((card) => {
        
        const numbersArrays = card[0].numbers;
        
        const flattenedNumbers = Array.from({ length: 25 }, (_, index) => {
          const arrayIndex = Math.floor(index / 5);
          const position = index % 5;
          
          return numbersArrays[arrayIndex][position];
        });
        const transformedArray = flattenedNumbers.map(value => value === 0 ? "free" : value);

        setBoard(transformedArray);
        setClicked(Array(25).fill(false))
      });
      
    } catch (error) {
      console.error("Error :", error);
    }
  };

  // Modified fetchCalledNumber to reset timers
  const fetchCalledNumber = async () => {
    try {
      bingo_on_chain_backend.get_game_state().then((val) => {
        
        setGameIsActive(val.is_active);
        let arrayNumbers = Array.from(val.called_numbers).slice(1);
        if(arrayNumbers.length > 0) {
          setCalledNumber(arrayNumbers[arrayNumbers.length-1]);
          setRecentCalls(arrayNumbers.slice(Math.max(0,arrayNumbers.length-7),arrayNumbers.length-1));
        }
      });
    } catch (error) {
      console.error("Error in fetchCalledNumber:", error);
    }
  };

  // Timer countdown effect
  useEffect(() => {
    const timerInterval = setInterval(() => {
      setTimeUntilNextCall((prev) => {
        if (prev <= 1) {
          fetchCalledNumber();
          return CALL_INTERVAL;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timerInterval);
  }, []);

  // Initial fetch
  useEffect(() => {
    fetchCalledNumber();
  }, []);

  // Your existing functions
  const checkCanChallenge = (clickedSquares) => {
    // Your existing challenge check logic
    for (let i = 0; i < 25; i += 5) {
      if (clickedSquares.slice(i, i + 5).every((square) => square)) return true;
    }
    for (let i = 0; i < 5; i++) {
      if ([0, 1, 2, 3, 4].every((j) => clickedSquares[i + j * 5])) return true;
    }
    if ([0, 6, 12, 18, 24].every((i) => clickedSquares[i])) return true;
    if ([4, 8, 12, 16, 20].every((i) => clickedSquares[i])) return true;
    return false;
  };

  const handleChallenge = async () => {
    setIsChecking(true);
    try {
      await bingo_on_chain_backend.challenge().then((val) => {
        if (val){
          setChallengeResult(val ? "success" : "failed");
          setGameIsActive(false)
          setCanChallenge(false)
          setIsChecking(false);
        }
      });
      setTimeout(() => {
        setChallengeResult(null);
      }, 3000);
    } catch (error) {
      console.error("Challenge verification failed:", error);
      setChallengeResult("failed");
    } finally {
      setIsChecking(false);
    }
  };

  const handleClick = (i) => {
    if (isChecking) return;
    const newClicked = [...clicked];
    newClicked[i] = !newClicked[i];
    setClicked(newClicked);
    setCanChallenge(checkCanChallenge(newClicked));
  };

  const renderHeader = (letter) => <div className="bingo-header">{letter}</div>;

  const renderSquare = (i) => {
    const isCalledNumber = board[i] === calledNumber;
    return (
      <button
        className={`bingo-square 
          ${clicked[i] ? "clicked" : ""} 
          ${challengeResult === "success" ? "winner" : ""}
          ${isChecking ? "checking" : ""}
          ${isCalledNumber ? "current-call" : ""}`}
        onClick={() => handleClick(i)}
        disabled={isChecking}
      >
        {board[i]}
      </button>
    );
  };

  return (
    <div className="bingo-game">
      {gameIsActive ?    <div className="called-numbers-display">
        <div className="current-call">
          <h2>Current Call</h2>
          <div className="number">{calledNumber}</div>
          <div className="timer">
            <div
              className="timer-bar"
              style={{ width: `${(timeUntilNextCall / CALL_INTERVAL) * 100}%` }}
            ></div>
            <div className="timer-text">
              Next number in: {formatTime(timeUntilNextCall)}
            </div>
          </div>
        </div>
        <div className="recent-calls">
          <h3>Recent Calls</h3>
          <div className="numbers">
            {recentCalls.slice(1).map((num, index) => (
              <div key={index} className="recent-number">
                {num}
              </div>
            ))}
          </div>
        </div>
      </div> : 
       <div >
       <div>
         <h1>Waiting for player</h1>
       </div>
     </div>
      }

      {challengeResult === "success" && (
        <div className="win-message">BINGO!</div>
      )}
      {challengeResult === "failed" && (
        <div className="fail-message">Not quite right!</div>
      )}
      <div className="bingo-board">
        <div className="header-row">
          {HEADERS.map((letter) => renderHeader(letter))}
        </div>
        {[0, 1, 2, 3, 4].map((row) => (
          <div key={row} className="board-row">
            {[0, 1, 2, 3, 4].map((col) => renderSquare(row * 5 + col))}
          </div>
        ))}
      </div>
      {canChallenge && !isChecking && !challengeResult && (
        <button className="challenge-button" onClick={handleChallenge}>
          Challenge!
        </button>
      )}
      {isChecking && (
        <div className="checking-message">Checking your bingo...</div>
      )}
      <button
        className="challenge-button"
        onClick={() => getCard()}
      >
        {gameIsActive ? "UnClick" :"Get Card!"}
      </button>
    </div>
  );
};

export default BingoGame;