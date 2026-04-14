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
        <div className="bottom-1/10 flex items-center absolute w-4/5">
            <textarea value={ value }
                className="resize-none w-3/5 flex items-center overflow-y-auto justify-center bg-txt/90 border max-h-35 text-primary placeholder:text-primary/50 py-1.5 px-4 rounded-lg"
                placeholder="Begin Your Course"
                rows={ 1 }
                onChange={handleChange}
            />
        </div>
    );
}

export default Search;