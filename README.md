
# git_rs

Current chapter 4.3

## TODO

- [ ] build release/patch
- [ ] 

https://stackoverflow.com/a/34797622

## Parsing commits without trying to kys

```regexp
(tree \w+\n)(parent \w+\n)*(author [a-zA-Z0-9<>@+. ]+\n)(committer [a-zA-Z0-9<>@+. ]+\n)(gpgsig (-+BEGIN PGP SIGNATURE-+\n(\n .+)+)-+END PGP SIGNATURE-+\n)?\n+([a-zA-Z0-9<>@+. #/]+\n*)+
```
