# shrekd [![CI](https://github.com/Nurrl/shrekd/actions/workflows/ci.yml/badge.svg)](https://github.com/Nurrl/shrekd/actions/workflows/ci.yml)
SHaRe, SHoRten oK Daemon, simple file, paste &amp; url sharing daemon

*Notice: This is still in early development and is not intended for production use right now.*

## Task list
- [x] Basic functionnality:
    - [x] File upload (**PUT** `/<filename>` *with streamed binary content*)
    - [x] Paste creation (**POST** `/paste`, *with streamed utf-8 content*)
    - [x] Url redirect creation (**POST** `/url` *with streamed url*)
    - [x] Getting record (**GET** `/<slug>`)
- [ ] Nice to have:
    - [x] Retain `file` records filenames and restore it at download
    - [x] Add the full path when returning the URL
    - [x] Retention curve depending on the weight, with expiration in return headers
    - [ ] Delete token in return headers, allowing *effort-less* deletion of a record
    - [x] CI:
        - [x] Setup CI for `cargo test`, `cargo clippy`
        - [x] Setup CI for `cargo audit`
        - [x] Setup CI for tag/release deployment
        - [x] Fix CI caching keys with `key` and `restore-keys`, cf. https://docs.github.com/en/actions/guides/caching-dependencies-to-speed-up-workflows#example-using-the-cache-action
    - [ ] User-programmable configuration:
        - [x] Custom Slug, best effort
        - [x] Slug length, minimum of sever-configured Slug
        - [x] Expiry date of the Record & Record detention duration
        - [x] Maximum download count
        - [ ] Input Checksum verification
        - [ ] Password-protected Records
    - [ ] UI on **GET** `/`:
        - [x] Make a dark/light mode compatible UI
        - [x] Get file creation working
        - [x] Get paste creation working
        - [x] Get url creation working
        - [ ] Get user parameters working with the above
        - [ ] Try to make a JS-free UI
- [ ] Reliability & Performance:
    - [x] Fix race conditions on files
    - [x] Serialize and deserialize data as binary, not JSON
    - [ ] Add unit tests to the project
    - [x] Take care of random slug collision
    - [ ] Server-side file encryption
    - [ ] Use redis transactions if relevant
    - [ ] Clean orphaned files at startup, if relevant and safe

## Abandonned task lists
- [ ] Abuse prevention:
    - [ ] Log input IP addresses in the record
    - [ ] Set up some kind of rate limiting by IP

    Can easily be done, with a higher reliability through a reverse proxy

## Setup

Currently the project requires you to host a **Redis** server locally for it to function properly.

### Setup with docker-compose

You will need [docker and docker-compose installed](https://docs.docker.com/compose/install/).

Once this is the case, you can run `shrekd` with:

```shell
$ docker-compose up -d
```

You can also modify the options of SHREKD with environment variable configuration by
editing the environment section of the `shrekd` service.

## Contributors

- Léon ROUX <Nurrl@users.github.com>
- Olivier Moreau <m-242@users.github.com>

## License

```
MIT License

Copyright (c) 2021 Léon ROUX

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
