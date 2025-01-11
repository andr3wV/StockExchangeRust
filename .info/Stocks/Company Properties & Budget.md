Better budgets will improve a company's image for **all agents** and the inverse for worse budgets.

Companies have a balance, basically their money.
Companies have hype, basically how many people are interested in the company's events
note: only 1-2 companies can have hype at any moment.
hype only lasts a short period.

> [!question] Question
> Should each *sector* of industry have 1 hype?


# Profit description
Companies have a profit rate, basically how much money they earn every \<insert unit of time\>

Earlier I was thinking companies would have a business model property (like ExtremelyProfitable, Profitable, Neutral, Terrible, ExtremelyTerrible). But I think property `profit_rate` covers it better.

Also it wouldn't be as simple as `balance += profit_rate` every tick, so a better name would be, `expected_profite_rate`.


# Liquid Loan? (idk if this is the correct term) (Company Event)
When companies need money, they can release shares.

How?
Let's say a company needs $10,000, their shares are currently priced at $10.
So they can sell 1000 shares (which they created out of a thin air).
The reason this strategy of creating money out of thin air if because people might not buy at 100, they might be like, we will buy only at 90.
OR if the demand is high, they might be able to sell at 110.
So the power is in the hands of the people

These transactions need to represented separately from normal transactions

> [!question] Why not have agents which act as representives for a company?
> I tried it and realized computing their preferences and holdings is wasteful, but to skip those, we would need to store and filter those agents from ALL the agents FOR EACH ITERATION.
> Ya this is expensive to do

# New company insertion (Company event)
The IPO (initial public offering) can be calculated from their `expected_profit_rate`.
BUT the price might rise due to higher hype.