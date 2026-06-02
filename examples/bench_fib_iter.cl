wea a = 0
wea b = 1
wea i = 0
mientras (i < 35) {
    wea tmp = b
    b = a + b
    a = tmp
    i = i + 1
}
lorea(a)