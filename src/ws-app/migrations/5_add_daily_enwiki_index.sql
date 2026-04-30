-- Initially adding the column as nullable
alter table edit_events
    add column meta_dt_date date;

-- Backfill the column from meta_dt field
-- WARNING: this will take 15+ minutes with the current database size (2026-04-30)
-- Ideally this should be done as a background job that updates the table in
-- chunks, but I don't have background jobs implemented yet.
update edit_events
set meta_dt_date = meta_dt::timestamptz::date
where meta_dt_date is null;

-- Require the column to be set in the future. Can't make it a generated column
-- because the timestamptz -> date conversion isn't immutable.
-- Note this breaks backwards compatibility. If I had a larger deployment,
-- multiple environments, or a more mature system, this migration should be
-- broken up into multiple steps so that existing ws-app deployments that don't
-- include the functionality to set this column would still work.
--
-- Since I'm still in early dev, it's okay to do a bit of cowboy DB admin.
alter table edit_events
    alter column meta_dt_date set NOT NULL;

-- Finally we can add the index that should massively speed up these queries
-- Really should be creating it concurrently but current migrations only run in
-- transactions, and you can't create an index concurrently in a transaction.
create index idx_edit_events_wiki_ns_date on edit_events ( wiki, namespace, meta_dt_date );