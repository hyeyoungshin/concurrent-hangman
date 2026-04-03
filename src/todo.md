**message passing**
- Voting Actor for session control
**event loop**
- get event loop to work






**shared state**
- (done) break down handle_client (double nested loop should be simplified)
  + (done) take voting mechanism out
- as soon as one of the players votes no, session break 
- (done) update player view every time there is a new guess

- players have different views
 - player 0 sees "guess a letter" and the view
 - player 1 sees just "guess a letter"

- When a player gets eliminated and the other guesses the secret word
  - the secret word is not revealed to the eliminated player
  - eliminated player is treated like it wasn't
  - the view does not update with the last blanked filled in


Q. Why stream.try_clone, not stream.clone?
try_clone returns Result type
file descriptor which is a unique number  
OS has a restricted handle on file desciptors 

Q Why Arc?
Arc vs Rc vs garbage collector

Be curious 
Be playful (make smallest example possible - good use of ai)

- do this for condvar
 - take votes
 - arbitrary client connecting
 - one no immediately closes voting