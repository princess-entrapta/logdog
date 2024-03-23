export function human_readable(num: number) {
    if (num > 1000000000) {
        return Number((num / 1000000000.0).toPrecision(3)) + "B"
    }
    if (num > 1000000) {
        return Number((num / 1000000.0).toPrecision(3)) + "M"
    }
    if (num > 1000) {
        return Number((num / 1000.0).toPrecision(3)) + "K"
    }
    return Number(num).toPrecision(3)
}