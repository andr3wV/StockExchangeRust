This article is going to be an eye roll for those who have made any simulations, but *we have to start somewhere*

When I started this project, I initially had a `HashMap<u64, Agent>` and a `HashMap<u64, Company>`, thinking that the constant lookup time would be fast enough for my purposes, I will also be able to store the Id as the key!

But thing went terribly bad as the function `get_mut_agent(hashmap, id)` and `get_agent(hashmap, id)` took **>20%** of the overhead of the program.

Clearly things had to change. And change they did!

Now I store each property in it's own 