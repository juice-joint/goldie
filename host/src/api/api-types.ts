export interface Song {
  name: string;
  uuid: string;
}

export interface FormattedSong extends Song {
  formattedName: string;
}

export interface ServerIpResponse {
  ip: string;
}
