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

## GitHub Actions Release Workflow Implementation

### User Request
Create GitHub Actions workflow to build releases for Linux, Mac OS and Windows. Please generate native installers where available. It would be nice to be able to manage versions easily. If it cannot be automated then please add a release management documentation explaining the release process.

### Task Plan
1. Create feature branch for release workflow
2. Save task plan to VIBE.md (this section)
3. Create comprehensive release workflow with:
   - Multi-platform builds (Linux x64, macOS x64/ARM64, Windows x64)
   - Native installers where possible (MSI for Windows, DMG for macOS, DEB/RPM for Linux)
   - Automated version management via git tags
   - Release artifact uploads
   - Release notes generation
4. Add automated version management system
5. Create release management documentation
6. Test the release workflow

### Implementation Details
- Use GitHub Actions matrix strategy for cross-platform builds
- Leverage rust-toolchain for consistent Rust versions
- Use cargo-wix for Windows MSI installer
- Use create-dmg for macOS DMG packages  
- Use cargo-deb and cargo-rpm for Linux packages
- Implement semantic versioning with git tags
- Auto-generate changelogs from commit messages
- Upload all artifacts to GitHub Releases

## GitHub Actions Cross-Compilation Fix - Issue #9

### User Request
Please check issue #9 on GitHub and see if you can fix it.

### Issue Analysis
GitHub issue #9 reported a failure in the GitHub Actions workflow during binary preparation:
- Error: `strip: Unable to recognise the format of the input file 'smart-crawler'`
- Occurred when building for ARM64 Linux (`aarch64-unknown-linux-gnu`) target
- Root cause: x86_64 `strip` command cannot process ARM64 binaries

### Task Plan
1. Create new branch for GitHub issue #9 fix
2. Update VIBE.md with task details (this section)
3. Fix cross-compilation strip issue in release workflow:
   - Install cross-compilation binutils for ARM64
   - Use target-specific strip tools (aarch64-linux-gnu-strip)
   - Add graceful fallback if strip tools unavailable
   - Enhanced error handling and logging
4. Commit the cross-compilation strip fix
5. Push branch for PR creation

### Implementation Details
- Added `binutils-aarch64-linux-gnu` package installation
- Implemented target-specific strip command selection:
  - ARM64 Linux: `aarch64-linux-gnu-strip`
  - x86_64 Linux: `strip`
  - macOS (both): `strip`
  - Windows: skip (not applicable)
- Added error checking and informative logging
- Updated RELEASE.md troubleshooting documentation

## Missing Installer Packages Fix

### User Request
I cannot see the msi, dmg, deb or rpm files in the latest releases even though there are steps in the release.yml GitHub Actions workflow. Can you please check and fix?

### Issue Analysis
The release workflow includes a `build-installers` job but the installer packages are not appearing in GitHub releases. Potential causes:
- Installer build job may be failing silently
- Dependencies for installer tools may be missing
- File paths or naming issues in the upload process
- Job dependencies or matrix configuration problems

### Task Plan
1. Create new branch for installer packages fix
2. Update VIBE.md with task details (this section)
3. Investigate missing installer packages in releases:
   - Review workflow logs for installer job failures
   - Check installer build steps and dependencies
   - Verify file paths and upload commands
4. Fix installer build workflow issues:
   - Ensure all installer tools are properly installed
   - Fix any path or dependency issues
   - Improve error handling and logging
5. Test and commit the fixes

### Expected Implementation
- Verify and fix Windows MSI installer generation
- Verify and fix macOS DMG package creation
- Verify and fix Linux DEB package generation  
- Verify and fix Linux RPM package generation
- Ensure all packages upload correctly to releases

## GitHub Actions PowerShell Syntax Fix - Issue #12

### User Request
Please check issue #12 on GitHub and see if you can fix it. Please use development workflow as in Claude.md

### Issue Analysis
GitHub issue #12 reports a GitHub Actions workflow failure with PowerShell syntax error:
- Error: "Missing expression after unary operator '--'"
- Occurs during "Build Release for Windows x64" job
- Command: `gh release upload v0.2.5 \ ... --repo "brainless/SmartCrawler"`
- Root cause: PowerShell multiline command continuation syntax issue

### Task Plan
1. Create new branch for GitHub issue #12 fix
2. Update VIBE.md with task details (this section)
3. Investigate PowerShell syntax error in release workflow:
   - Examine the problematic gh release upload command
   - Identify line continuation and escaping issues
   - Check shell context (bash vs PowerShell)
4. Fix the gh release upload command syntax:
   - Ensure proper PowerShell multiline syntax
   - Use correct shell for the command execution
   - Test command syntax compatibility
5. Test the fix and commit changes

### Implementation Details
- The error occurs because PowerShell interprets backslash differently than bash
- Need to either use PowerShell-compatible line continuation or force bash shell
- Will use explicit shell specification to ensure consistent behavior
- May need to restructure the command for better cross-platform compatibility

## Cargo-deb Asset Path Fix - Issue #14

### User Request
Can you please check issue #14 on GitHub and see if you can fix it. Please use development workflow as in Claude.md

### Issue Analysis
GitHub issue #14 reports a GitHub Actions workflow failure during DEB package building:
- Error: "TOML parse error at line 29, column 10"
- Message: "Please only use `target/release` path prefix for built products"
- Location: `Cargo.toml` DEB package configuration
- Root cause: Using hardcoded cross-compilation path instead of standard path

### Task Plan
1. Create new branch for GitHub issue #14 fix
2. Update VIBE.md with task details (this section)
3. Investigate cargo-deb TOML parse error:
   - Examine the problematic asset path in Cargo.toml
   - Understand cargo-deb path requirements
   - Identify cross-compilation compatibility issues
4. Fix DEB package asset path in Cargo.toml:
   - Change from target-specific path to standard target/release
   - Let cargo-deb handle cross-compilation paths automatically
   - Ensure compatibility with GitHub Actions workflow
5. Test the fix and commit changes

### Implementation Details
- The issue is in the [package.metadata.deb] assets configuration
- Current path: "target/x86_64-unknown-linux-gnu/release/smart-crawler"
- Required path: "target/release/smart-crawler"
- cargo-deb will automatically handle cross-compilation target paths
- This maintains compatibility while following cargo-deb best practices

## Cargo-rpm File Path Fix - Issue #16

### User Request
Can you please check issue #16 on GitHub and see if you can fix it. Please use development workflow as in Claude.md

### Issue Analysis
GitHub issue #16 reports a GitHub Actions workflow failure during RPM package building:
- Error: "cp: cannot stat 'target/x86_64-unknown-linux-gnu/rpm/RPMS/x86_64/*.rpm': No such file or directory"
- Location: GitHub Actions workflow RPM build step
- Root cause: cargo-rpm may not generate files in the expected target-specific path
- Issue occurs when trying to copy the generated RPM file

### Task Plan
1. Create new branch for GitHub issue #16 fix
2. Update VIBE.md with task details (this section)
3. Investigate cargo-rpm build and file location issue:
   - Examine the problematic file copy path in workflow
   - Research cargo-rpm default output locations
   - Understand how cargo-rpm handles cross-compilation paths
4. Fix RPM package file path in GitHub Actions workflow:
   - Update workflow to look in correct cargo-rpm output directory
   - Add better error handling and file discovery
   - Ensure compatibility with cargo-rpm behavior
5. Test the fix and commit changes

### Implementation Details
- Current failing path: "target/x86_64-unknown-linux-gnu/rpm/RPMS/x86_64/*.rpm"
- cargo-rpm likely generates files in "target/rpm/" or "target/generate-rpm/"
- Need to investigate actual cargo-rpm output structure
- Will add file discovery logic to handle different cargo-rpm versions
- May need to use glob patterns or find commands for robustness

## Crawler Improvements Implementation - Issue #19

### User Request
Can you please check ideas in #19 on GitHub and implement them. Please use development workflow as in Claude.md

### Issue Analysis
GitHub issue #19 suggests several improvements to enhance the SmartCrawler's URL handling and LLM interaction:

1. **URL Tracking Enhancement**: Track URLs per domain storing only path and query parameters
2. **LLM Interaction Improvement**: Send only path/query to LLM, handle domain separately
3. **Objective-based URL Expansion**: Add URLs matching original objective words after filtering

### Task Plan
1. Create new branch for GitHub issue #19 improvements
2. Update VIBE.md with task details (this section)
3. Implement URL tracking improvements in SmartCrawler:
   - Modify URL storage to track path+query per domain
   - Update deduplication logic for improved efficiency
4. Update LLM prompt to handle domain separately:
   - Modify LLM trait default implementation
   - Update prompt to receive domain context separately from URL paths
5. Add objective-based URL expansion logic:
   - Implement keyword matching against objective
   - Add URLs that match objective terms after select_urls_one_level_deeper
6. Test and commit the improvements

### Implementation Details
- URL storage: Use HashMap<String, HashSet<String>> where key=domain, value=path+query
- LLM prompt: Pass domain context separately, send clean path/query list
- Objective matching: Parse objective for keywords, match against URL paths
- Performance: Reduce memory usage by storing partial URLs instead of full URLs
- Accuracy: Improve LLM decisions with cleaner, domain-contextualized prompts

## Browser Scrolling Enhancement Implementation - Issue #21

### User Request
Can you please check ideas in #21 on GitHub and implement them. Please use development workflow as in Claude.md

### Issue Analysis
GitHub issue #21 requests an enhancement to the `Browser::scrape_url` method to improve content capture:

1. **Add scrolling functionality**: Before extracting HTML content, scroll through the page
2. **Mimic human behavior**: Scroll at a realistic pace that resembles human browsing
3. **Time limitation**: Stop scrolling after 10 seconds maximum
4. **Dynamic content capture**: Help load JavaScript-rendered content that appears on scroll

### Task Plan
1. Create new branch for GitHub issue #21 improvements
2. Update VIBE.md with task details (this section)
3. Implement browser scrolling functionality in Browser::scrape_url:
   - Add scroll-before-extract logic to scrape_url method
   - Implement realistic scrolling behavior using fantoccini WebDriver commands
   - Add 10-second time limit with appropriate timing controls
4. Add realistic scrolling with proper timing and behavior:
   - Scroll incrementally to mimic human behavior
   - Use reasonable delays between scroll actions
   - Handle pages of different lengths gracefully
5. Test scrolling implementation
6. Commit and push the improvements

### Implementation Details
- Use fantoccini WebDriver scroll commands (execute_script with window.scrollBy)
- Implement incremental scrolling with delays (e.g., scroll 300px every 500ms)
- Track total scroll time to enforce 10-second limit
- Detect when page bottom is reached to avoid unnecessary scrolling
- Ensure graceful handling of scroll failures or timeouts
- Log scrolling progress for debugging and monitoring

## Typed Entity Extraction Implementation

### User Request
"In the crawler, I want to get data that is typed. So if objective needs people, then we should give a struct that is People with first and last name, etc. Please create data types for common entities that describe real world entities like People, Date, Event, Location, etc."

### Task Plan
1. Create new branch for typed entity extraction feature
2. Update VIBE.md with task details (this section)
3. Analyze current codebase structure and data handling approach
4. Create comprehensive entity data types for common real-world entities:
   - Person (name, title, company, contact info)
   - Location (address, coordinates, venue details)
   - Event (title, dates, location, organizer, pricing)
   - Product (name, price, brand, specifications, reviews)
   - Organization (company details, industry, employees)
   - NewsArticle (headlines, content, publication info)
   - JobListing (position details, salary, employment type)
5. Integrate typed entities with LLM analysis results:
   - Extend LLM trait with entity extraction method
   - Add structured JSON extraction with confidence scoring
   - Implement fallback to original text analysis
6. Update crawler to use typed results:
   - Modify CrawlResult to include extracted entities
   - Enhance console output for entity display
   - Maintain backward compatibility
7. Test compilation and fix any errors
8. Commit and push the improvements

### Implementation Details
- All entities implement Serialize, Deserialize, Debug, Clone, PartialEq
- ExtractedEntity enum provides type-safe entity variants  
- EntityExtractionResult tracks extraction metadata and confidence (0.0-1.0)
- Type-safe accessor methods: get_persons(), get_locations(), etc.
- Structured JSON prompts for reliable entity extraction
- Enhanced console output with entity counts and confidence scores
- JSON output includes structured entity data alongside original analysis
- Added chrono dependency for DateTime handling
- Fixed clippy warnings for code quality

### Files Modified
- `src/entities.rs` - New comprehensive entity definitions
- `src/lib.rs` - Added entities module export
- `src/llm.rs` - Added extract_entities method to LLM trait
- `src/crawler.rs` - Integrated entity extraction into crawl workflow
- `src/main.rs` - Enhanced console output for entity display
- `Cargo.toml` - Added chrono dependency for DateTime support

### Benefits
- Structured, typed data extraction instead of unstructured text
- Type safety for entity handling and processing
- Extensible entity system for future entity types
- Confidence scoring for extraction quality assessment
- Backward compatibility with existing analysis workflow

## Enhanced Entity Extraction with TypeScript Schema Prompts

### User Request
"Looks great! Now let us change our web page analysis prompt and ask LLM that if it finds data as per the objective then it should return JSON that conforms to the data types we have. We may have to generate TypeScript types from the Rust types."

### Task Plan
1. Create new branch for enhanced entity extraction with TypeScript types
2. Update VIBE.md with task details (this section)
3. Generate TypeScript type definitions from Rust entities:
   - Create comprehensive TypeScript interfaces matching Rust structs
   - Include all entity types, enums, and supporting structures
   - Add proper type annotations and optional fields
4. Update LLM prompts to reference TypeScript schemas:
   - Replace generic entity structure examples with precise TypeScript definitions
   - Provide complete schema for each entity type
   - Include validation requirements and field constraints
5. Integrate enhanced prompts into entity extraction:
   - Update extract_entities method with new schema-aware prompts
   - Improve JSON parsing validation
   - Enhance error handling for schema conformance
6. Test and commit the improvements

### Implementation Details
- Generate TypeScript interfaces for all Rust entity types
- Create comprehensive schema documentation in prompts
- Use TypeScript syntax for better LLM understanding of expected JSON structure
- Include enum values, optional fields, and nested object schemas
- Maintain backward compatibility with existing entity extraction
- Improve extraction accuracy through precise schema definitions

### Expected Benefits
- Higher accuracy in entity extraction through precise schema guidance
- Better JSON structure conformance from LLM responses
- Reduced parsing errors and validation issues
- Clearer documentation of expected data structures
- Enhanced type safety in extracted data

### Additional Improvements
- Updated analyze_content method to also use TypeScript schemas
- Unified schema-aware approach across both entity extraction and content analysis
- Enhanced fallback analysis with entity-structured output formatting
- Consistent data presentation following TypeScript entity organization

### JSON Parsing Robustness Fix
- Fixed "EOF while parsing a string" JSON parsing errors
- Added robust JSON extraction from LLM responses that may contain extra text
- Improved error handling with detailed logging for debugging
- Enhanced prompt clarity to discourage markdown formatting in JSON responses
- Added comprehensive tests for JSON extraction edge cases

## Improve URL Selection Logic - Issue #27

### User Request
Implement improved URL selection logic that uses LLM to generate keywords based on the crawling objective, then ranks URLs by keyword relevance before LLM selection.

### Issue Analysis
For websites with large numbers of URLs from sitemaps or homepages, the current approach sends too many URLs to the LLM for selection. This can be inefficient and may not yield the best results.

### Task Plan
1. Create new branch for GitHub issue #27 improvements
2. Update VIBE.md with task details (this section)
3. Add new LLM method `generate_keywords()` to extract relevant keywords from objective
4. Implement URL scoring and ranking system based on keyword matches:
   - Score URLs by keyword matches in path, query parameters, and URL structure
   - Sort URLs by relevance score before sending to LLM
   - Limit URLs sent to LLM to top X candidates (where X = max_urls * multiplier)
5. Integrate two-stage URL selection process into crawler:
   - Stage 1: Keyword-based filtering and ranking
   - Stage 2: LLM-based final selection from top candidates
6. Add configuration options for keyword-based filtering parameters
7. Test with various objectives and URL sets
8. Ensure backward compatibility with existing URL selection logic

### Implementation Details
- `generate_keywords()` method: Extract 5-10 relevant keywords from crawling objective
- URL scoring algorithm: Score based on keyword matches, path depth, and structure
- Ranking system: Sort URLs by score, take top N candidates for LLM selection
- Two-stage selection: Keyword filtering → LLM selection for higher quality results
- Configuration: Add multiplier setting for candidate URL limit (e.g., max_urls * 3)

### Expected Benefits
- Better URL selection quality through objective-specific keyword pre-filtering
- Reduced LLM API calls and costs by sending fewer, more relevant URLs
- Faster crawling with more targeted URL selection
- Improved handling of large sitemaps and homepage link collections
- Maintained accuracy while improving efficiency

## One Pager Product Website - Issue #29

### User Request
Create a simple one pager website in a `website` directory using Vite and small libraries for static hosting.

### Issue Analysis
Need to create a product landing page that:
- Uses Vite as build tool with minimal, lightweight libraries
- Designed for static site hosting
- Describes SmartCrawler product based on README content
- Includes semantic HTML structure with proper navigation
- Features hero section with screenshot placeholder
- Contains demo video section with YouTube placeholder

### Task Plan
1. Create new branch for GitHub issue #29
2. Update VIBE.md with task details (this section)
3. Set up Vite project in `website` directory:
   - Initialize with minimal dependencies
   - Configure for static site generation
   - Choose lightweight CSS framework or vanilla CSS
4. Create semantic HTML structure:
   - Navigation menu for major sections
   - Hero section with CTA and screenshot placeholder
   - Demo video section with YouTube embed placeholder
   - Feature sections based on README content
   - Installation/usage instructions
   - Footer with links
5. Style the website:
   - Modern, clean design
   - Responsive layout for mobile/desktop
   - Professional appearance suitable for developer tool
6. Test and optimize for static hosting
7. Commit and push the website

### Implementation Details
- **Build Tool**: Vite for fast development and optimized builds
- **Libraries**: Minimal dependencies, prefer vanilla JS/CSS or lightweight alternatives
- **Content**: Extract product description and features from README.md
- **Structure**: Single page with smooth scrolling navigation
- **Hosting**: Optimized for static site deployment (GitHub Pages, Netlify, etc.)
- **Assets**: Placeholder images and demo video for initial version

### Expected Deliverables
- Complete one-page website in `website/` directory
- Vite configuration for static builds
- Semantic HTML with accessibility considerations
- Responsive CSS styling
- Navigation menu linking to page sections
- Hero section with product overview and screenshot placeholder
- Demo video section with YouTube placeholder
- Installation and usage instructions
- Professional appearance suitable for open source project

## GitHub Issue #31: Allow passing in root URL as domain

### User Request
Allow passing "http://example.com" or "https://example.com/" as the domain argument in CLI. Trailing slash should be optional. Please extract domain from them.

### Task Plan
1. ✅ Create new branch for GitHub issue #31
2. ✅ Save user request and task plan to VIBE.md  
3. Analyze current domain parsing logic in CLI args
4. Implement URL-to-domain extraction function
5. Update CLI argument parsing to handle full URLs
6. Add tests for URL domain extraction
7. Test with various URL formats
8. Commit and push changes

### Implementation Details
- Need to handle both HTTP and HTTPS URLs
- Remove trailing slashes automatically
- Extract just the domain part from full URLs
- Maintain backward compatibility with existing domain-only inputs
- Add proper error handling for malformed URLs

## GitHub Issue #34: Treat homepage as the first URL

### User Request
In our crawler::crawl_domain, we collect URLs from the homepage but we do not analyze the homepage content. Instead I would like to treat the homepage as the first URL in our analysis loop. When sitemap is empty, the homepage is the first and only URL to be analyzed. We do not need to rank or select URLs with LLM if there is only one URL. In the loop, we should find new URLs. These new URLs should go through our ranking and LLM based selection strategy. Thus, in the next iteration of the loop, we select the best candidate considering the page we just analyzed.

### Task Plan
1. ✅ Create new branch for GitHub issue #34
2. ✅ Save user request and task plan to VIBE.md
3. Analyze current crawl_domain logic and homepage handling
4. Modify crawl_domain to treat homepage as first URL to analyze
5. Update URL analysis loop to start with homepage when sitemap is empty
6. Skip LLM selection when only one URL (homepage)
7. Ensure new URLs from homepage go through ranking/LLM selection
8. Add tests for the new homepage-first logic
9. Run formatters, linters and tests
10. Commit and push changes

### Implementation Details
- Homepage should be the first URL analyzed instead of just source for links
- When sitemap is empty, start analysis loop with homepage URL
- Skip LLM/ranking when only one URL (just homepage)
- New URLs discovered from homepage should go through full ranking/LLM selection
- Maintain iterative crawling where each analyzed page contributes URLs for next iteration
- Ensure homepage content is properly analyzed and entities extracted

## GitHub Issue #32: Allow passing links to extract data

### User Request
I would like to pass links to pages with a `--links` argument. There are two modes that it can affect the crawler:

1. Without --domains argument: In this mode, the crawler should not fetch sitemap or URLs of other pages. Instead it should only try and extract data from the given URLs in the --links argument
2. With --domains argument: In this mode the crawling starts from the URLs given in --links argument instead of the homepage. Sitemap extraction should happen but URLs found in the pages provided in the --links argument should be added to the total list of links to analyze.

### Task Plan
1. ✅ Create new branch for GitHub issue #32
2. ✅ Save user request and task plan to VIBE.md
3. Analyze current CLI structure and add --links argument
4. Implement mode 1: --links without --domains (extract only from given URLs)
5. Implement mode 2: --links with --domains (start crawling from given URLs)
6. Update crawl_domain logic to handle starting URLs
7. Add tests for both modes of --links functionality
8. Add input validation for URL formats in --links
9. Run formatters, linters and tests
10. Commit and push changes

### Implementation Details
- Add new `--links` CLI argument that accepts comma-separated URLs
- Mode 1 (links-only): Skip sitemap discovery, skip domain crawling, analyze only provided URLs
- Mode 2 (links + domains): Use provided URLs as starting points instead of homepage, include sitemap URLs
- URLs in --links should be validated and parsed using existing domain extraction logic
- Need to handle URLs from different domains gracefully in links-only mode
- Maintain backward compatibility with existing domain-only workflow
- Provide clear error messages for invalid URL formats or conflicting arguments

## GitHub Issue #37: Change clean-html mode to work with given URL instead of local path

### User Request
As a user I would like to use the --clean-html mode in the CLI to use our crawler to fetch the given URL if we detect a URL instead of local path in the first parameter. The second parameter should continue to be the local output path.

### Task Plan
1. ✅ Save user request and task plan to VIBE.md
2. ✅ Create new branch for the task (feature/clean-html-url-support)
3. ✅ Analyze current --clean-html implementation
4. ✅ Modify CLI args to detect URL vs local path for --clean-html
5. ✅ Implement URL fetching in clean-html mode
6. ✅ Add tests for URL detection and fetching
7. ✅ Run formatters, linters and tests
8. ✅ Commit and push to new branch

### Implementation Details
- Need to detect if first parameter is a URL (starts with http:// or https://) vs local file path
- If URL detected, fetch the content using browser/WebDriver instead of simple HTTP client
- Second parameter remains the output path for cleaned HTML
- Maintain backward compatibility with existing local file functionality

### Status: ✅ COMPLETED
- Successfully implemented URL support using browser/WebDriver
- Added crypto provider initialization for browser operations
- All tests passing (97/97)
- Feature works with both local files and URLs

## GitHub Issue #39: Improve HTML cleaner

### User Request
I would like the HTML cleaner to remove images unless:
- The image has a name that looks like a slug of readable text
- The image has an alt text

I would like the HTML cleaner to remove all HTML comments.

### Task Plan
1. Save user request and task plan to VIBE.md
2. Create new branch for the task (feature/improve-html-cleaner)
3. Analyze current HTML cleaner image handling
4. Implement image filtering logic (slug name or alt text)
5. Implement HTML comment removal
6. Add tests for new image and comment filtering
7. Run formatters, linters and tests
8. Commit and push to new branch

### Implementation Details
- **Image Filtering**: Remove `<img>` tags unless they have:
  - A `src` attribute with a filename that looks like a slug (readable text with hyphens/underscores)
  - An `alt` attribute with meaningful text
- **Comment Removal**: Remove all HTML comments `<!-- ... -->`
- **Preserve Existing Functionality**: Keep all current HTML cleaning behavior
- **Add Tests**: Test image filtering with various scenarios and comment removal
