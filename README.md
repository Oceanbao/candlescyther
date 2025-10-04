# candlescyther

Reap long candles.

## Requirement

- auto 1W KD monitor:
  - hot theme tickers (quantum, crypto LIST, etc.)
  - top ones, by vol by cap
- auto 1D KD monitor:
  - custom add by user, batch add
- design KD metric:
  - derivative of MACD nearing dip, what's the behaviour, study
  - do it for 1W and 1M
- User can search ticker and get 1W chart with company meta

### Resource

**1D**
<https://54.push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35106707668456928451_1695010059469&secid=105.TSLA&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=101&fqt=1&beg=0&end=20500101&lmt=1200&_=1695010059524>

**1W**
<https://push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35105424247560587396_1758630789935&secid=105.APP&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=102&fqt=1&beg=0&end=20500101&smplmt=755&lmt=1000000&_=1758630789945>

<https://push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35105424247560587396_1758630789935&secid=105.APP&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=102&fqt=1&end=20500101&lmt=120&_=1758630789946>

**IB Data Science**
<https://www.interactivebrokers.com/campus/category/ibkr-quant-news/data-science/>

### Steps

```sh
v0.1.0
Back
- [x] setup web server with infra
- [x] create simple log db, migration
- [ ] git up, deploy to fly
Front
- shell page, maybe a template from somewhere
- deploy to CF page
```

- design schema
- create migration for tables
- create route "/new/[ticker]" to run through db and crawl
- one time job to add watch list: crawl, compute, store
- add route "/kd"
- add route "/price/[ticker]"
- add CRON
- OpenAPI
- FRONT

## Backend Service

- "/new/[ticker]" GET
  - verify ticker
  - if ok:
    - crawl ticker info, market cap, key meta for sorting
    - store meta
    - crawl weekly prices
    - store prices, compute macd/kd
    - return "prices" so FRONT shows chart
  - else:
    - show error
- CRON: every weekend Friday night, crawl all tickers weekly price
  - stores them (optimize db)
  - compute MACD, KD, store (another table)
- "/kd" GET returns all KD and MACD scores
- "/price/[ticker]" GET returns prices (front re-compute macd/kd)
- CRON: auto scan some hot list 1W price and store and compute (make a route)
- FRONT: show list, show chart
- LOG: create a log table tracks cron jobs, errors, etc. UI page show logs

```sh
QUERY: show all ripe 7K sorted.
- go to db, get all tickers sorted by 7K and take KD < 30
- make `[KD]` send to response

QUERY: does ticker exist
- request with ticker, search db, if yes response 1W data to chart, else return no,
- UI shows option to add new ticker

COMMAND: create new ticker to track (assigner queue?)
- crawl ticker 1W data -> `WeekClose` and `TickerMeta`
- save to db
- return 201
- UI shows success
```

## Frontend UI

### Main page

- landing, dashboard style
