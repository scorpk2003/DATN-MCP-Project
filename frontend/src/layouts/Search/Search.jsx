import React, { useState } from "react";
function Search() {
    let [value, Setvalue] = useState("");
    const handleChange = (e) => {
        const e1 = e.target;
        e1.style.height = "auto";
        e1.style.height = e1.scrollHeight + "px";
        Setvalue(e1.value);
    }
    return (
        <div className="bottom-1/10 flex items-center absolute w-full p-8 mx-7">
            <textarea value={ value }
                className="resize-none w-3/5 flex items-center overflow-y-auto justify-center bg-txt/80 border-none max-h-35 text-primary placeholder:text-primary/50 py-1.5 px-4 rounded-lg"
                placeholder="Begin Your New Course"
                rows={ 1 }
                onChange={handleChange}
            />
        </div>
    );
}

export default Search;