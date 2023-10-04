
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

033041dc248e52fd864caa62f9fac167f338b3a4 -> with gpgsig
781992a8eaf46bc202dbd5ac5205e33e43e4e92c -> without gpgsig

https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object
