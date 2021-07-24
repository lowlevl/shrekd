# shrt
SHare and shoRTen, simple file, paste &amp; url sharing service

*Notice: This is still in early development and is not intended for production use right now.*

## Task list
- [ ] User-programmable configuration:
    - [x] Custom Slug, best effort
    - [x] Slug length, minimum of sever-configured Slug
    - [x] Expiry date of the Record & Record detention duration
    - [x] Maximum download count
    - [ ] Input Checksum verification
- [x] File upload (**POST** `/file` *with streamed input*)
- [ ] Redirect creation (**POST** `/redirect`)
- [ ] Paste creation (**POST** `/paste`)
- [x] Getting record (**GET** `/<slug>`)
- [ ] Retention curve depending on the weight, with expiration in return headers
- [ ] Delete token in return headers, allowing deletion of a record

## Setup

Currently the project requires you to host a **Redis** server locally for it to function properly.

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
