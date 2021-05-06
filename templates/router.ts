/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

/** Removes trailing slashed from URI */
const normalizeUri = (uri: string) => {
  uri = uri.trim();
  if (uri.length == 0) {
    throw new Error('uri is empty');
  }

  let uriLength = uri.length;
  if (uri[uriLength - 1] == '/') {
    uri = uri.slice(0, uriLength - 1);
  }
  return uri;
};

/** URI<-> Fn mapping type */
type routeTuple = {
  uri: string;
  fn: () => void;
};

/**
 * Router that selectively executes fucntions
 * based on window.location.pathname
 * */
export class Router {
  routes: Array<routeTuple>;
  constructor() {
    this.routes = [];
  }

  /**
   * registers a route-function pair with Router
   * @param {string} uri - route to be registered
   * @param {function} fn: - function to be registered when window.locatin.path
   * matches uri
   * */
  register(uri: string, fn: () => void) {
    uri = normalizeUri(uri);

    if (
      this.routes.find(route => {
        if (route.uri == uri) {
          return true;
        }
      })
    ) {
      throw new Error('URI exists');
    }

    const route: routeTuple = {
      uri,
      fn,
    };
    this.routes.push(route);
  }

  /**
   * executes registered function with route
   * matches window.pathname.location
   * */
  route() {
    const path = normalizeUri(window.location.pathname);

    let fn: () => void | undefined;

    this.routes.forEach(route => {
      const pattern = new RegExp(`^${route.uri}$`);
      if (path.match(pattern)) {
       fn =  route.fn;
      }
    });

    if (fn === undefined) {
    throw new Error("Route isn't registered");
    }

    return fn();
  }
}
