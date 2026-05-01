# RFC-02: Database Reduction

**Status**: Draft

## Context

Ingesting all the events into the database is causing the database size to grow
faster than the size of the raw eventstream data. This growth is going to
exhaust the disk space available on the dev machine Wikistream is running on
soon, and will make it much harder to move Wikistream to my Kubernetes host.

Let's calculate how much "runway" we have before Wikistream runs out of disk
space.

Today is 2026-04-30. The first full day of stream data is from 2026-04-18.
That's 13 full days, inclusive of today.

- Current size of raw stream data: 60 GiB
  * ~5 GiB/day
- Current size of wikistream database: 115 GiB
  * ~9 GiB/day
- Total size: 175 GiB
  * ~13.5 GiB/day

The machine running Wikistream right now has 500 GiB of disk space. Assuming an
allocation of 100 GiB for everything else on the machine, we need to know how
long it'll take Wikistream to hit 400 GiB of disk usage.

- Disk space buffer remaining = 400 GiB - 175 GiB = 225 GiB
- Days of disk space remaining = 225 GiB / 13.5 GiB/day = ~16 days

Changes are needed to reduce the data growth rate. I need more time to get more
disk capacity online. I'll also be able to work on other changes to Wikistream
that could potentially reduce disk usage even further.

## Recommendation

Remove any data from the database that isn't used by the only feature currently
in Wikistream.

1. Drop the `categorize_events`, `log_events`, and `new_events` tables from the
   database.
2. Drop all rows from the `edit_events` database, except for those where the
   `wiki` column is `'enwiki'`.
3. Update ws-app's `/ingest` endpoint to discard any event other than `edit`
   events in `'enwiki'`.
4. Update ws-sse-cli's ingest command to skip any event other than `edit`
   events in `'enwiki'`.

We can estimate how much additional runway this would give us by estimating how
much space that data is currently taking up in the database.

- Size of the `edit_events` table: 79 GiB
  * Reported by `\d+` in Postgres console
- Number of rows in `edit_events`: ~16,000,000
- Number of rows in `edit_events` in the `'enwiki'` wiki: ~2,000,000 (12.5%)
- Size of `edit` events in the `'enwiki'` wiki (est.): 79 GiB * 0.125 = ~10 GiB
  * ~0.8 GiB/day
- Estimated usage: ~6 GiB/day
- Estimated days of disk space remaining = 225 GiB / 6 GiB/day = ~37 days

That's an extra 21 days, and most of the growth is from the raw event data
which should be fairly easy to compress and archive elsewhere.

This should make queries faster because there won't be a ton of unused data in
the `edit_events` table that Postgres will have to skip over. Ingest should
also be faster because the majority of the events will be skipped.

## Potential Consequences

This will make it harder to add new features that require data that we are no
longer ingesting to the database. We'd have to update the database schema again
and run a lengthy backfill of all historical event data first.

## Alternatives Considered

There's a few other things that could be done to save disk space. I'm not
implementing these right now, but I'll likely implement them in the future.

- Dropping unused columns in the `edit_events` table.
- Re-evaluating column data types. I suspect several of the columns could be
  stored using smaller types with no loss of data.
- Compressing and archiving event data. I'm still storing all the event data
  files in their raw form on disk. It'll save a lot of space (and make it
  easier to manage the data) if those files get consolidated, compressed, and
  possibly archived to a different location. ws-sse-cli's ingest command (or a
  successor to it) would need to be updated to support this archival format.

Before I make many more database changes, I want to implement a system of
running background jobs as an alternative to executing long-running database
migration queries within SQLx migrations. Once that's available, the database
schema changes above will be easier to implement.
