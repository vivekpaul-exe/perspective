/******************************************************************************
 *
 * Copyright (c) 2017, the Perspective Authors.
 *
 * This file is part of the Perspective library, distributed under the terms of
 * the Apache License 2.0.  The full license can be found in the LICENSE file.
 *
 */

import {PRIVATE_PLUGIN_SYMBOL} from "../model";
import {save_column_size_overrides} from "../model/column_overrides.js";

/**
 * Serialize the state of this plugin to a token.
 *
 * @returns This plugin's state as a token.
 */
export function save() {
    if (this.regular_table) {
        const datagrid = this.regular_table;
        const token = {
            columns: {},
            scroll_lock: !!this._is_scroll_lock,
            editable: !!this._is_edit_mode,
        };

        for (const col of Object.keys(datagrid[PRIVATE_PLUGIN_SYMBOL] || {})) {
            const config = Object.assign(
                {},
                datagrid[PRIVATE_PLUGIN_SYMBOL][col]
            );
            if (config?.pos_fg_color || config?.pos_bg_color) {
                config.pos_fg_color = config.pos_fg_color?.[0];
                config.neg_fg_color = config.neg_fg_color?.[0];
                config.pos_bg_color = config.pos_bg_color?.[0];
                config.neg_bg_color = config.neg_bg_color?.[0];
            }

            if (config?.color) {
                config.color = config.color[0];
            }

            token.columns[col] = config;
        }

        const column_size_overrides = save_column_size_overrides.call(this);

        for (const col of Object.keys(column_size_overrides || {})) {
            if (!token.columns[col]) {
                token.columns[col] = {};
            }

            token.columns[col].column_size_override =
                column_size_overrides[col];
        }

        return JSON.parse(JSON.stringify(token));
    }
    return {};
}
