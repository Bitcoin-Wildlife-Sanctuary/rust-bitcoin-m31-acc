# Batch multiplication for M31 and its extension

This repository is experimenting a way to perform a large number of M31 computation in a batch in a way to save overhead.

This idea is from Avihu Levy from StarkWare.

Liam Eagen from Alpen Labs was the first to point out doing multiplication of $a$ and $b$ through the squares of
$a+b$ and $a-b$ and the side effect that the result is multiplied by 4 can be simply addressed by pre-floor-dividing 
each element in the lookup table by 4 where it can be proven that doing so still yields the correct result and removes
the side effect. The proof is to observe that $(a+b)^2$ and $(a-b)^2$ must be congruent modulo 4. The proof can be trivial 
as by showing $(a+b)^2 - (a-b)^2 = 4ab$ and $4ab$ is congruent to 0 modulo 4.

A more detailed writeup is provided here: https://hackmd.io/@l2iterative/Byg8h1MsC

### License and contributors

This repository is intended to be public good. It is under the MIT license. 

A portion of the code is contributed by [L2 Iterative (L2IV)](https://www.l2iterative.com/), a crypto
VC based in San Francisco and Hong Kong. The work receives support from Starkware, who is a limited partner in L2IV. For
disclosure, L2IV has also invested into numerous companies active in the Bitcoin ecosystem, but this work is open-source
and nonprofit, and is not intended for competition. The code is not investment advice.

There are also community members contributing to the code and contributing to the ideas. Bitcoin Wildlife Sanctuary is a
public-good project supported by many people.

Below we reiterate the contributors to this repository.

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Liam-Eagen"><img src="https://avatars.githubusercontent.com/u/5618692?v=4?s=100" width="100px;" alt="Liam Eagen"/><br /><sub><b>Liam Eagen</b></sub></a><br /><a href="#research-Liam-Eagen" title="Research">🔬</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->