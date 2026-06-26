import { useState } from "react";
import { Composer } from "../../components/ui";

function Search({ placeholder = "Bạn muốn học gì hôm nay?", onSubmit }) {
  const [value, setValue] = useState("");

  const handleSubmit = () => {
    if (!value.trim()) {
      return;
    }

    onSubmit?.(value);
    setValue("");
  };

  return (
    <Composer
      value={value}
      onChange={(event) => setValue(event.target.value)}
      onSubmit={handleSubmit}
      placeholder={placeholder}
      submitLabel="Tạo lộ trình"
    />
  );
}

export default Search;
