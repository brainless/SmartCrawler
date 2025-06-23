# This project was mostly vibe coded

## First version was created with Claude Code
### Initial prompt

> Please create a crawler and scraper in Rust. You can use any crates you want. The crawler will be given an objective and list of domains via command prompt. Objective may ask for existence of some information or list of extracted data or similar. The crawler should find the sitemap for any domain and ask Claude which URLs make most sense to crawl for the given objective. The crawler should be conservative in   crawling and try to use Claude to reach results fast.

The first version did not handle when sitemap file is not found, so I tried:

> If a sitemap file is not found for a domain, can you generate a SiteMap from the links hierarchy found from the homepage or other pages? Then refer to this SiteMap as you continue crawling and base your decisions on it.

This did not work well, the way Claude wrote the code was to first crawl a bunch of URLs to generate a possible sitemap. It does not work, not sure why but it would also defeat the goal of a conservative crawl.

Here are the next prompts I tried:

> Regarding CrawlerConfig.domains, will it work if domains get added while the domains are being crawled in the loop in SmartCrawler::crawl_all_domains?

> Can we update CrawlerConfig.domains so that domains can be added while existing domains are being crawled in the loop in SmartCrawler::crawl_all_domains?

> Please modify ClaudeClient::select_urls to accept urls of type SitemapUrl or just regular URLs.

> In SmartCrawler keep track of all the URLs that are being scraped for each domain in the function crawl_domain. Only unique URLs allowed.

> In SmartCrawler::crawl_domain if sitemap_urls.is_empty then we should not return immediately. Instead we should use the root URL for the domain and start with it.

> In ClaudeClient::select_urls please update prompt to only return URLs that in the existing list of urls argument. Also please check the response from Claude and ignore return URLs that are not in the existing list of urls.

> In ClaudeClient::select_urls please add an info log of the urls argument.

> In SmartCrawler::crawl_domain if sitemap_urls.is_empty then we should take the root url of the domain and scrape that URL to get all the URLs in it.
