dataFile = open("ffmpeg4.txt", "r")
lines = dataFile.readlines()

denominator = 0
finalNumber = 0
av = 0
for line in lines:
    number = int(line)
    finalNumber = finalNumber + number
    denominator = denominator + 1

print(denominator)
print(len(lines))

print(finalNumber / denominator)
