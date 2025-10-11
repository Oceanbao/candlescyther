# candlescyther

Reap long candles.

## Requirement

- auto 1W KD monitor:
  - hot theme tickers (quantum, crypto LIST, etc.)
  - top ones, by vol by cap
- auto 1D KD monitor:
  - custom add by user, batch add
- design KD metric:
  - 1W KD 30 alert
  - derivative of MACD D-Line nearing dip, approaching zero, when to alert
- User can search ticker and get 1W chart with company meta

### Resource

**1D**
<https://54.push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35106707668456928451_1695010059469&secid=105.TSLA&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=101&fqt=1&beg=0&end=20500101&lmt=1200&_=1695010059524>

**1W**
<https://push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35105424247560587396_1758630789935&secid=105.APP&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=102&fqt=1&beg=0&end=20500101&smplmt=755&lmt=1000000&_=1758630789945>

<https://push2his.eastmoney.com/api/qt/stock/kline/get?cb=jQuery35105424247560587396_1758630789935&secid=105.APP&ut=fa5fd1943c7b386f172d6893dbfba10b&fields1=f1%2Cf2%2Cf3%2Cf4%2Cf5%2Cf6&fields2=f51%2Cf52%2Cf53%2Cf54%2Cf55%2Cf56%2Cf57%2Cf58%2Cf59%2Cf60%2Cf61&klt=102&fqt=1&end=20500101&lmt=120&_=1758630789946>

**meta**
<https://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=1&cb=jQuery35105571137681219451_1708499614785&fields=f57%2Cf58%2Cf107%2Cf162%2Cf152%2Cf167%2Cf92%2Cf59%2Cf183%2Cf184%2Cf105%2Cf185%2Cf186%2Cf187%2Cf173%2Cf188%2Cf84%2Cf116%2Cf85%2Cf117%2Cf190%2Cf189%2Cf62%2Cf55&secid=105.TSLA&ut=fa5fd1943c7b386f172d6893dbfba10b&wbp2u=%7C0%7C0%7C0%7Cweb&_=1708499614786>

```go
type RawStockCrawl struct {
 Data struct {
  EPS                float64 `json:"f55"`
  Ticker             string  `json:"f57"`
  Name               string  `json:"f58"`
  Market             int     `json:"f107"`
  TotalShare         float64 `json:"f84"`
  TotalShareOut      float64 `json:"f85"`
  NetAssetPerShare   float64 `json:"f92"`
  NetProfit          float64 `json:"f105"`
  TotalCap           float64 `json:"f116"`
  TradeCap           float64 `json:"f117"`
  PricePerEarning    float64 `json:"f162"`
  PricePerBook       float64 `json:"f167"`
  ROE                float64 `json:"f173"`
  TotalRevenue       float64 `json:"f183"`
  TotalRevenueChange float64 `json:"f184"`
  NetProfitChange    float64 `json:"f185"`
  GrossProfit        float64 `json:"f186"`
  ProfitMargin       float64 `json:"f187"`
  DebtRatio          float64 `json:"f188"`
  DateOfPublic       int     `json:"f189"`
  UndistProfit       float64 `json:"f190"`
 } `json:"data"`
}

```

**IB Data Science**
<https://www.interactivebrokers.com/campus/category/ibkr-quant-news/data-science/>

**Yahoo Price**
<https://query1.finance.yahoo.com/v8/finance/chart/app?interval=1d&period1=1704085200&period2=1759363200&lang=en-US&region=US>
(1m, 2m, 5m, 15m, 30m, 60m, 90m, 1h, 4h, 1d, 5d, 1wk, 1mo, 3mo)

```go
type RawDailyCrawl struct {
 Chart struct {
  Result []struct {
   Meta struct {
    Symbol string `json:"symbol"`
   } `json:"meta"`
   Timestamp  []int `json:"timestamp"`
   Indicators struct {
    Quote []struct {
     Open   []float64 `json:"open"`
     High   []float64 `json:"high"`
     Close  []float64 `json:"close"`
     Low    []float64 `json:"low"`
     Volume []float64 `json:"volume"`
    } `json:"quote"`
   } `json:"indicators"`
  } `json:"result"`
  Error struct {
   Code        string `json:"code"`
   Description string `json:"description"`
  } `json:"error"`
 } `json:"chart"`
}

```

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

## Changelog

```sh
v0.1.0
-------------------------------------
Back
- [x] setup web server with infra
- [x] create simple log db, migration
- [x] git up, deploy to fly
Front
- [x] shell page, maybe a template from somewhere
- [x] deploy to CF page

v0.2.0
-------------------------------------
Backend
- [ ] schema: stock meta (cap, vol, eps, etc., kd, macd, ma)
- [x] schema: 1W data (OHLC)
- [x] schema: job (type, status, detail)
- [x] migration
- [ ] fn: crawl_meta(ticker) -> Meta (log)
- [x] fn: crawl_price(ticker) -> 1WPrice (log)
- [x] fn: create_jobs(TJob) -> Result<(), Error>
- [x] fn: run_jobs(TJob) -> Result<(), Error>
- [x] test above
Front
- [ ] logs page
- [ ] jobs page
- [ ] tickers info (pagination) (table) (click to see price chart)

v0.3.0
-------------------------------------
Backend
- [ ] route GET "/new/[ticker]" COMMAND runs through crawls and store (async) (log)
- [ ] route GET "/jobs" QUERY with param to get job log
- [ ] route GET "/kd" QUERY with param to get kd table
- [ ] route GET "/price/[ticker]" QUERY to get price table per ticker
- [ ] cron: for all tickers, crawl_price()
- [ ] openAPI all routes
Front
```
