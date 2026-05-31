-- Removing a ton of columns that we don't use to save a bunch of space. May
-- consider adding some of these back in the future if needed for certain
-- features.
alter table edit_events
    drop column schema,
    drop column comment,
    drop column bot,
    drop column server_url,
    drop column server_name,
    drop column server_script_path,
    drop column parsedcomment,
    drop column meta_uri,
    drop column meta_domain,
    drop column meta_stream,
    drop column meta_topic,
    drop column meta_partition,
    drop column meta_offset,
    drop column id,
    drop column notify_url,
    drop column minor,
    drop column length,
    drop column revision
;