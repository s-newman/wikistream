# Wikistream

Wikistream is a link aggregator-style website that shows the daily top 25 most edited articles on the English
Wikipedia. It consumes the [Wikimedia EventStreams] `recentchanges` data feed, ingest relevant events to a Postgres
database, and has a rudimentary web interface to explore each day's most edited articles.

[Wikimedia EventStreams]: https://wikitech.wikimedia.org/wiki/Event_Platform/EventStreams_HTTP_Service

Wikistream is still in early development and the website is not yet publicly available. Development happens in a 
private GitLab repository (because I'm more comfortable using and self-hosting GitLab CI) and the Wikistream
application is currently only deployed on a computer in my closet. This repo is just a mirror of the code for others
to browse at their leisure.

Check back later for updates on a public instance of Wikistream once it's ready!