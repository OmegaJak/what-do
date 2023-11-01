# What Do?
A simple website that helps a group answer "what to do" through a voting process.

This website guides a group through a voting process aimed at finding the best compromise when balancing competing interests/dislikes among a group of friends trying to decide how to spend their time.

## The Voting Process
1. It starts with someone creating a 'room' where the voting will take place, seeded with initial options.
2. Then it progresses to the veto/add options stage, where anyone can join the voting process, add options they want to be considered, and veto options they're unwilling to do. All of this is synchronized live for all users present in the room. Once everyone has submitted their options for consideration and finished vetoing, voting progresses to ranking.
3. Each user then ranks all options in the order of how much they prefer them
4. The votes are tallied and results are calculated. A score is generated for each option based on the rankings it received. The winning option should be something that everyone is at least somewhat happy to do.

<img src="https://github.com/OmegaJak/omegajak.github.io/blob/gh-pages/Misc/WhatDo/landing_page.jpg" width=400)><img src="https://github.com/OmegaJak/omegajak.github.io/blob/gh-pages/Misc/WhatDo/veto_page.jpg" width=400)><img src="https://github.com/OmegaJak/omegajak.github.io/blob/gh-pages/Misc/WhatDo/rank_page.jpg" width=400)>
<img src="https://github.com/OmegaJak/omegajak.github.io/blob/gh-pages/Misc/WhatDo/results_page.jpg" width=400)>

## Credits
- Built using David Peterson's [Axum Live View](https://github.com/davidpdrsn/axum-live-view) for live SSR
- Styled using Kev Quirk's excellent [Simple.css](https://simplecss.org/)
- Deployed easily and for free using [shuttle.rs](https://www.shuttle.rs/)
