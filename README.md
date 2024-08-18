# Batch multiplication for M31 and its extension

This repository is experimenting a way to perform a large number of M31 computation in a batch in a way to save overhead.

This idea is from Avihu Levy from StarkWare.

Liam Eagen from Alpen Labs was the first to point out doing multiplication of $a$ and $b$ through the squares of
$a+b$ and $a-b$ and the side effect that the result is multiplied by 4 can be simply addressed by pre-floor-dividing 
each element in the lookup table by 4 where it can be proven that doing so still yields the correct result and removes
the side effect. The proof is to observe that $(a+b)^2$ and $(a-b)^2$ must be congruent modulo 4. The proof can be trivial 
as by showing $(a+b)^2 - (a-b)^2 = 4ab$ and $4ab$ is congruent to 0 modulo 4.