# Trackerconqueror
Random scraper i felt like writing for modarchive, ended up being a useful journey to learn about
networking and whatever, also gave birth to [marchive-open-db](https://github.com/phnixir/marchive-open-db)
which is cool, wear your hazmat suits before going sourcecode diving, a rust beginner wrote this
after all.

## Notes
Always make sure the port that the server will bind to is open to the network,
modern linux distributions and those who aren't minimal usually do this on their own temporarily
and then close it after the program is done, but linux distros like 4MLinux or TheSSS don't.
This caused me immense pain and cost me 5 hours to find out, fuck TheSSS, why can't you open ports
automatically
