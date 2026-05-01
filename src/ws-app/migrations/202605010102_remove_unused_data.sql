-- Removing unused data per RFC-02
-- Pre-requisite: removing ingest code for unused event types
drop table categorize_events;
drop table log_events;
drop table new_events;

-- This index is no longer needed since all rows will be in the same wiki
drop index idx_edit_events_wiki_ns_date;

-- Dropping all edits from other wikis
delete
from edit_events
where wiki != 'enwiki';