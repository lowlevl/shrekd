# shrtd
SHare and shoRTen Daemon, simple file, paste &amp; url sharing daemon

*Notice: This is still in early development and is not intended for production use right now.*

## Task list
- [ ] User-programmable configuration:
    - [x] Custom Slug, best effort
    - [x] Slug length, minimum of sever-configured Slug
    - [x] Expiry date of the Record & Record detention duration
    - [x] Maximum download count
    - [ ] Input Checksum verification
- [x] File upload (**POST** `/file` *with streamed input*)
- [x] Redirect creation (**POST** `/redirect`)
- [x] Paste creation (**POST** `/paste`)
- [x] Getting record (**GET** `/<slug>`)
- [ ] Retention curve depending on the weight, with expiration in return headers
- [ ] Delete token in return headers, allowing deletion of a record
- [ ] Add the full path when returning the URL

- [ ] Reliability
    - [x] Fix race conditions on files
    - [ ] Use redis transactions if relevant

## Setup

Currently the project requires you to host a **Redis** server locally for it to function properly.

### Setup with docker-compose

You will need [docker and docker-compose installed](https://docs.docker.com/compose/install/).

Once this is the case, you can run `shrtd` with:

```shell
$ docker-compose up -d
```

By default, it will run on port 8000 and store data in `/tmp`, but you can use these variables to change the defaults:
- `SHRTD_DATADIR` // Datadir's path (default /tmp)
- `SHRTD_PORT`    // Port exposed (default 8000).

You can also modify the options of SHRTD with environment variable configuration by
editing the environment section of the shrtd service.

## Contributors

- Léon ROUX <Nurrl@users.github.com>

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
