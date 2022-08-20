/******************************************************************************
 *
 * Copyright (c) 2017, the Perspective Authors.
 *
 * This file is part of the Perspective library, distributed under the terms of
 * the Apache License 2.0.  The full license can be found in the LICENSE file.
 *
 */

import {PRIVATE_PLUGIN_SYMBOL} from "../../model";

import {cell_style_numeric} from "./numeric.js";
import {cell_style_string} from "./string.js";
import {cell_style_datetime} from "./datetime.js";
import {cell_style_boolean} from "./boolean.js";
import {cell_style_row_header} from "./row_header.js";

function get_psp_type(metadata) {
    if (metadata.x >= 0) {
        return this._column_types[metadata.x];
    } else {
        return this._row_header_types[metadata.row_header_x - 1];
    }
}

export function table_cell_style_listener(regularTable) {
    const plugins = regularTable[PRIVATE_PLUGIN_SYMBOL] || {};

    for (const tr of regularTable.children[0].children[1].children) {
        for (const td of tr.children) {
            const metadata = regularTable.getMeta(td);
            const column_name =
                metadata.column_header?.[metadata.column_header?.length - 1];

            let type = get_psp_type.call(this, metadata);
            const plugin = plugins[column_name];
            const is_numeric = type === "integer" || type === "float";

            if (is_numeric) {
                cell_style_numeric.call(this, plugin, td, metadata);
            } else if (type === "boolean") {
                cell_style_boolean.call(this, plugin, td, metadata);
            } else if (type === "string") {
                cell_style_string.call(this, plugin, td, metadata);
            } else if (type === "date" || type === "datetime") {
                cell_style_datetime.call(this, plugin, td, metadata);
            } else {
                td.style.backgroundColor = "";
                td.style.color = "";
            }

            td.classList.toggle(
                "psp-bool-type",
                type === "boolean" && metadata.user !== null
            );

            const is_th = td.tagName === "TH";
            if (is_th) {
                cell_style_row_header.call(this, regularTable, td, metadata);
            }

            td.classList.toggle("psp-align-right", !is_th && is_numeric);
            td.classList.toggle("psp-align-left", is_th || !is_numeric);
            td.classList.toggle(
                "psp-color-mode-bar",
                plugin?.number_fg_mode === "bar" && is_numeric
            );
        }
    }
}
