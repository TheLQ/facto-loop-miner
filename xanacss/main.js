function offset(el) {
    const rect = el.getBoundingClientRect(),
        scrollLeft = window.scrollX || document.documentElement.scrollLeft,
        scrollTop = window.scrollY || document.documentElement.scrollTop;
    return { top: rect.top + scrollTop, left: rect.left + scrollLeft }
}

addEventListener("load", () => {
    let val = offset(document.getElementsByClassName("unit-beacon")[2])
    console.log("val", val)
})