"use client"
import styles from "./page.module.css";
import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap-icons/font/bootstrap-icons.css";
import { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { emit } from '@tauri-apps/api/event';

export default function Home() {
  const names = ["全体", "一般", "耳打ち", "チーム", "クラブ", "システム", "叫び"];
  const init = useRef(false);
  const labelRefs = [
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
    useRef((null as unknown) as HTMLLabelElement),
  ];
  const divRefs = [
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
    useRef((null as unknown) as HTMLDivElement),
  ];
  const spacerRef = useRef((null as unknown) as HTMLDivElement);
  const [messages, setMessages] = useState<[string, string, string][][]>([...Array(names.length)].map(_ => []));
  const [views, setViews] = useState([...Array(names.length)].map(_ => true));
  const [verbose, setVerbose] = useState(false);
  const [wrap, setWrap] = useState("soft");
  const [vertical, setVertical] = useState(true);
  const [autoScroll, setAutoScroll] = useState([...Array(names.length).map(_ => true)]);

  useEffect(() => {
    const resizeViewImpl = () => {
      const view_count = views.filter((e: any) => e).length;
      for (let i = 0; i < views.length; ++i) {
        if (!views[i]) {
          continue;
        }
        if (vertical) {
          const height = (window.innerHeight - labelRefs[i].current.offsetHeight * view_count - spacerRef.current.offsetHeight) / view_count;
          divRefs[i].current.style.height = height + "px";
        } else {
          const height = window.innerHeight - labelRefs[i].current.offsetHeight - spacerRef.current.offsetHeight;
          divRefs[i].current.style.height = height + "px";
        }
      }
    };
    const resizeView = (event: Event) => resizeViewImpl();
    resizeViewImpl();
    window.addEventListener("resize", resizeView);

    const f = async () => {
      await listen('read', async event => {
        setMessages(event.payload as [[string, string, string][]]);
      });
      for (let i = 0; i < names.length; ++i) {
        await listen('view' + i, async event => {
          const views = await invoke("get_views") as [boolean, boolean, boolean, boolean, boolean, boolean, boolean];
          setViews(views);
        });
      }
      type State = { verbose: boolean, wrap: string, vertical: boolean, auto_scroll: boolean[] };
      await listen('verbose', async event => {
        const state = await invoke("get_state") as State;
        setVerbose(state.verbose);
      });
      await listen('wrap', async event => {
        const state = await invoke("get_state") as State;
        setWrap(state.wrap);
      });
      await listen('vertical', async event => {
        const state = await invoke("get_state") as State;
        setVertical(state.vertical);
      });
      const views = await invoke("get_views") as [boolean, boolean, boolean, boolean, boolean, boolean, boolean];
      setViews(views);
      const state = await invoke("get_state") as State;
      setVerbose(state.verbose);
      setWrap(state.wrap);
      setVertical(state.vertical);
      setAutoScroll(state.auto_scroll);
    };
    if (!init.current) {
      console.log('init')
      init.current = true;
      f();
    }

    return () => {
      console.log('clear');
      window.removeEventListener("resize", resizeView);
    };
  }, [vertical, views, autoScroll]);

  useEffect(() => {
    for (let i = 0; i < names.length; ++i) {
      if (!views[i] || !autoScroll[i]) {
        continue;
      }
      if (divRefs[i].current !== null) {
        divRefs[i].current.scrollTop = divRefs[i].current.scrollHeight;
      }
    }
  }, [messages]);

  const toggleAutoScroll = async (event: React.MouseEvent<HTMLDivElement>) => {
    const i = parseInt(event.currentTarget.id[event.currentTarget.id.length - 1]);
    const state = !autoScroll[i];
    setAutoScroll(prev => prev.map((e, j) => i === j ? !e : e));
    await emit(event.currentTarget.id, { auto_scroll: state });
  };

  return (
    <div className="container-fluid">
      {vertical && names.map((name, i) => {
        return views[i] && (
          <div key={name}>
            <label className={`pt-2 form-label ${styles["view-label"]}`} ref={labelRefs[i]}>{name}</label>
            <span className={`ms-3 ${autoScroll[i] ? "" : "opacity-25"}`} onClick={toggleAutoScroll} id={"auto_scroll" + i}>
              <i className="bi bi-card-text text-light"></i>
              <i className="bi bi-arrow-down-short text-light"></i>
            </span>
            <div className={`formcontrol ${styles.view}`} style={{overflow: "auto"}} ref={divRefs[i]}>
              {
                messages[i].map((e, j) => {
                  const message = verbose ? e[2] + " " + e[0] : e[0];
                  const style = wrap === "soft" ? {color: e[1]} : {color: e[1], whiteSpace: "nowrap"};
                  return <div key={j} style={style}>{message}</div>;
                })
              }
            </div>
          </div>
        );
      })}
      {!vertical &&
        <div className={`row row-cols-${views.filter(e => e).length}`}>
          {names.map((name, i) => {
            return views[i] && (
              <div className="col" key={name}>
                <label className={`pt-2 form-label ${styles["view-label"]}`} ref={labelRefs[i]}>{name}</label>
                <div className={`form-control ${styles.view}`} style={{overflow: "auto"}} ref={divRefs[i]}>
                  {
                    messages[i].map((e, j) => {
                      const message = verbose ? e[2] + " " + e[0] : e[0];
                      const style = wrap === "soft" ? {color: e[1]} : {color: e[1], whiteSpace: "nowrap"};
                      return <div key={j} style={style}>{message}</div>;
                    })
                  }
                </div>
              </div>
            );
          })}
        </div>
      }
      <div className="invisible p-1" ref={spacerRef}></div>
    </div>
  );
}
