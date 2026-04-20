# RFC-01: Resume without duplicates

**Status**: Draft

## Context

When `ws-sse-cli` resumes from local files, it'll always duplicate at least one event.

When it reads the latest event to determine the last event ID, it uses the timestamp of the latest event as the 
event ID. However, testing shows that passing a timestamp to EventStreams in the `Last-Event-ID` header will cause 
EventStreams to start with the event at that specific timestamp. This means the first event `ws-sse-cli` receives on 
resume will be the event it has _just read from disk_ to determine the last event ID (in almost all cases).

## Recommendation

When `ws-sse-cli` resumes from local files, it will also read a unique identifier from the latest event.

[//]: # (TODO: what unique identifier? `.id` isn't unique or even always set)

Once it starts receiving events from EventStreams, `ws-sse-cli` will skip all events until it receives an event 
matching the event it read from disk. That event will also be skipped. The next event and all future events will be 
handled normally.

## Potential Consequences

If `ws-sse-cli` **never** receives an event matching the latest event it read from disk, it will continue skipping 
events forever.

## Alternatives Considered

### Increment Timestamp

In testing, it seems like EventStreams simply increments the timestamp in the `.meta.dt` field by one to get the
`.timestamp` value in the `id: ` field it emits for each event. This does ensure that EventStreams resumes 
immediately _after_ the event in question.

However, if multiple events have the exact same timestamp, incrementing the timestamp could potentially cause 
`ws-sse-cli` to miss data. This isn't theoretical; the test data has observed up to 4 separate events all with the 
same `.meta.dt` timestamp.

For example, let's say two events have the exact same timestamp: events A and B. The following event, C, has a later 
timestamp.

Let's pretend `ws-sse-cli` stops execution after receiving event A, but before receiving event B.

If `ws-sse-cli` increments the timestamp it reads from event A, then the first event it will receive from 
EventStreams is event C. Event B will be entirely missed!

I haven't dug through the sample raw stream data to check how EventStreams handles repeated `.meta.dt` timestamps 
when creating the `id: ` field.

[//]: # (TODO: figure out how EventStreams handles repeated `.meta.dt` timestamps when creating the `id: ` field)