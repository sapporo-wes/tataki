`cwltool` の実行

```bash
docker run -it --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -v "$PWD":"$PWD" -w "$PWD" quay.io/commonwl/cwltool:3.1.20230906142556 --help
```

sam.cwl の help

```bash
docker run -it --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -v "$PWD":"$PWD" -w "$PWD" quay.io/commonwl/cwltool:3.1.20230906142556 ./sam.cwl --help
```

sam.cwl の実行

```bash
docker run -it --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -v "$PWD":"$PWD" -w "$PWD" quay.io/commonwl/cwltool:3.1.20230906142556 ./sam.cwl --input_file ./toy.sam

...

INFO [job sam.cwl] completed success
{
    "result": true
}INFO Final process status is success
```

or (using remote file)

```bash
docker run -it --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -v "$PWD":"$PWD" -w "$PWD" quay.io/commonwl/cwltool:3.1.20230906142556 ./sam.cwl --input_file https://raw.githubusercontent.com/samtools/samtools/develop/examples/toy.sam
```

sam.cwl に fasta file を入力したら

```bash
docker run -it --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -v "$PWD":"$PWD" -w "$PWD" quay.io/commonwl/cwltool:3.1.20230906142556 ./sam.cwl --input_file https://raw.githubusercontent.com/samtools/samtools/develop/examples/toy.fa
INFO [job sam.cwl] completed success
{
    "result": false
}INFO Final process status is success
```
