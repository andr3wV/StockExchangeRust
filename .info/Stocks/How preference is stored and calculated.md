My first intuition was a `HashMap<u64, u64>` where the key is the company Id and the value is just a preference number, to get the probability to just divide it with the `vec.sum()`.

While the performance isn't terrible, the function `get_preferred_random` already creates the highest overload. But while sleeping I can up with a better approach.

I would have a list of 100 items; where every tick, a company_id will be added to the list, while also removing the first item, so the size is still 100.

example with list of 10 items and only 2 companies
```
| 2 | 1 | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 |
```
Then whenever we want a random preferred company, we get just a random item from the list. The more occurrences of a company occurs, the more preferred it will be.

Here, 2 occurs 6 times, in a list of 10 items, so when we get a random item. The probability of company 2 to be picked would be **0.6**.
Note that I had to do no calculations to find the probability.

Let's say company 1 comes up with a better budget and our agent, now likes company 1.
```
Adding 1 to the list
| 2 | 1 | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 |


| 1 | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 | 1 |
```
Now, after their better result, company 1 has improved their image for our agent has equal preference for both companies.

Another advantage about this approach is that, old performances matter less than current performances as they will eventually be popped off the list.

This was the gist of it. Now some implementation details for those who want to know

# Technicalities

## Improved appending
Instead of popping of the list, we just keep a track of which index is the target for removal.
And when we get a new item, we just replace the target item and increment the target index
e.g.
```
| 2 | 1 | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 |
  ^  target index

New item comes: 1

| 1 | 1 | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 |
      ^ target index

New item comes: 1

| 1 | 1 | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 |
          ^ target index

New item comes: 1

| 1 | 1 | 1 | 1 | 2 | 2 | 2 | 1 | 2 | 1 |
              ^ target index
```
This is WAY more faster for obvious reasons.

## No particular bias conditions
Also sometimes the agent doesn't have a particular bias for a specific iteration.
So I have 2 approaches to fix this issue
- instead of adding a single number, add a `HashMap<u64, u64>` with a small set of companies which performed and how well they performed
- instead of adding a single number, add multiple!. I would probably make the preference list 1000 items long, and add 10 items each iteration.

## Negative preference
Another thing to think about is when a company does terrible, we want to drop the preference of that company, that would be hard thing to do, because we need to iterate through the entire list and find and remove that company id.

A better idea would be to store preferred company id along with preferred action!
So let's say company 1 does well, we add `(1, Buy)`, if it does terrible, we add `(1, Sell)`.

## Recency bias
Another advantage to this solution will be my ability to add recency bias, instead of looking at the last 1000 items, look at the last 100 items!.

Now of course it wouldn't be **as easy**, considering our improved appending approach. But still very computationally cheap