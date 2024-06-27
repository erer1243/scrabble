f = open("./wordlist-20210729.txt", "r")
f2 = open("./dequoted.txt", "w")

while word := f.readline():
    dequoted = word.strip().strip("\"")
    f2.write(dequoted)
    f2.write("\n")

f.close()
f2.close()
