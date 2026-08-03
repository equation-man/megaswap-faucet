# MegaSwap Trader Simulation Token Issuer  
This is megaswap-faucet, the token issuer for megaswap trading simulation environment. The protocol distributes trading tokens 
to the simulation environment wallets dynamically.  
It distributes larger amount of tokens when the pool is overflowing beyond a particular threshold 
but automatically throttles the distribution to limit token dispense if the faucet supply drops below a particular level.  

This protocol leverages Proportional Controller(P Controller) mechanism. In control theory, proportional controller 
changes its output dynamically based on the size of the error. In this design, the error is inverted. 
Instead of calculating how much supply we are missing, output to be issued is scaled based on how much 
surplus token supply the faucet has left. The protocol also has automint guard rail to fire when tokens dips below a critical level to increase the supply.

