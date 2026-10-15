import DOMPurify from "dompurify"
import { h, ref, useId, type VNode } from "vue"
import ChatImage from "../components/shared/chat/ChatImage.vue"
import ChatExternalLink from "../components/shared/chat/ChatExternalLink.vue"

export const ALLOWED_TAGS = ["a", "b", "blockquote", "br", "code", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "i", "img", "li", "ol", "p", "pre", "strong", "sub", "sup", "table", "tbody", "td", "tfoot", "th", "thead", "tr", "ul"]
export const ALLOWED_ATTR = ["href", "title", "colspan", "rowspan", "dir", "class"]

/**
 * Parser is used to convert chunks of plain irc messages into interactive UI
 * elements. Such as links, embed images or mentions. If messag does not contain
 * any URLs or flagged elements, string goes out. Otherwise a vue render functions
 * render components in their place
 */

const URL = /(?<!["'=])\bhttps?:\/\/[^\s<>"'`]+(?<![.,;:!?)\]}'"])/gi
const IMAGE = /(gif|jpe?g|tiff?|png|webp|bmp)$/i
const VIDEO = /(mp4|webpm)$/i
const AUDIO = /(mp3|wav|ogg)$/i

type Child = VNode | string

// Converts URL into a DOM element (or vue component) handler
function urlToNode(url: string, galleryId: string): Child {
  // TODO: mentions
  // TODO: [internal] server links
  // TODO: [internal] channel links
  // TODO: [internal] voice links
  // TODO: link embeds (og image / title etc)
  // TODO: spotify embeds (create a custom provider?)
  // TODO: custom video player
  // TODO: custom audio player
  if (IMAGE.test(url)) {
    // Append a gallery id to all images within a gallery
    return h(ChatImage, { src: url, "data-gallery-id": galleryId })
  }

  if (AUDIO.test(url)) {
    return h("audio", { controls: true, src: url })
  }

  const video = url.match(VIDEO)
  if (video) {
    return h("video", { controls: true }, [h("source", { src: url, type: `video/${video[0]}` })])
  }

  // Default back to <a /> if link was not caught or doesn't have a custom
  // component assosicated to it yet
  return h(ChatExternalLink, { href: url }, () => url)
}

// Splits string into either plain text nodes or parsed vue components
function splitText(text: string): Child[] {
  const pieces: Child[] = []
  let last = 0

  // Create an id for all images within a single message. Now, when gallery
  // modal is open, we can collect the images (1-n <img> elements) and get their
  // src to create a gallery
  const galleryId = useId()

  // Extracts URLs out of a string and either returns empty string nodes or
  // components retrieved from urlToNode
  for (const match of text.matchAll(URL)) {
    if (match.index > last) {
      // If a normal string was between the last URL and this
      pieces.push(text.slice(last, match.index))
    }

    pieces.push(urlToNode(match[0], galleryId))

    // Save the end of URL index for inbetween string
    last = match.index + match[0].length
  }

  // gap between last URL and end of the text
  if (last < text.length) {
    pieces.push(text.slice(last))
  }

  return pieces
}

// Sanitizes the message. Return it as a node hierarchy that is then walked and
// parsed based on if the node is just text node or not
export function parseMessage(text?: string | null): Child[] {
  if (!text) return []

  const sanitized = DOMPurify.sanitize(text, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
  })

  return splitText(sanitized)
}

// Track opened modal handlers for links / images etc. The idea is to ship
// global components so that message components do _not_ embed and import any
// components themselves
interface MessageModal {
  type: "img" | "link"
  data: any
}

export const messageModalState = ref<MessageModal | null>(null)

export function openMsgModal(type: MessageModal["type"], data: MessageModal["data"]) {
  messageModalState.value = { type, data }
}

export function closeMsgModal() {
  messageModalState.value = null
}

export function isMsgActive(type: MessageModal["type"]) {
  return !!(messageModalState.value && messageModalState.value.type === type && !!messageModalState.value.data)
}
